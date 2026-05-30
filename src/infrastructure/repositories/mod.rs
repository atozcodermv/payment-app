use crate::{
    application::dto::*,
    domain::repositories::*,
    domain::services::invoice_service::ensure_transition,
    domain::value_objects::invoice_state::InvoiceState,
    infrastructure::{auth::argon::{hash_api_key, verify_api_key}, psp::{client::charge, models::ChargeRequest}},
    shared::errors::{AppError, AppResult},
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepo {
    pub pool: PgPool,
}

#[derive(Debug)]
pub struct WebhookDelivery {
    pub event_id: Uuid,
    pub url: String,
    pub secret: String,
    pub payload: Value,
}

impl PostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

async fn invoice_response(pool: &PgPool, business_id: Uuid, invoice_id: Uuid) -> AppResult<InvoiceResponse> {
    let row = sqlx::query("SELECT id, customer_id, state, currency, amount_due_cents, amount_paid_cents, created_at, updated_at FROM invoices WHERE business_id = $1 AND id = $2")
        .bind(business_id)
        .bind(invoice_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("invoice_not_found"))?;
    let items = sqlx::query_as::<_, (Uuid, String, i32, i32)>("SELECT id, description, quantity, unit_amount_cents FROM line_items WHERE invoice_id = $1 ORDER BY created_at")
        .bind(invoice_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(id, description, quantity, unit_amount_cents)| LineItemResponse { id, description, quantity, unit_amount_cents })
        .collect();
    Ok(InvoiceResponse {
        id: row.try_get("id")?,
        customer_id: row.try_get("customer_id")?,
        state: row.try_get("state")?,
        currency: row.try_get("currency")?,
        amount_due_cents: row.try_get("amount_due_cents")?,
        amount_paid_cents: row.try_get("amount_paid_cents")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        line_items: items,
    })
}

pub mod invoice_transitions {
    use super::*;

    pub async fn transition(pool: &PgPool, business_id: Uuid, invoice_id: Uuid, next: &str) -> AppResult<InvoiceResponse> {
        let mut tx = pool.begin().await?;
        let row = sqlx::query("SELECT state FROM invoices WHERE business_id = $1 AND id = $2 FOR UPDATE")
            .bind(business_id)
            .bind(invoice_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound("invoice_not_found"))?;
        let current: String = row.try_get("state")?;
        let next_state = InvoiceState::parse(next);
        ensure_transition(&current, next_state)?;
        sqlx::query("UPDATE invoices SET state = $1, updated_at = now() WHERE id = $2")
            .bind(next)
            .bind(invoice_id)
            .execute(&mut *tx)
            .await?;
        if next == "open" {
            enqueue_webhook_tx(&mut tx, business_id, "invoice.created", json!({"invoice_id": invoice_id, "state": "open"})).await?;
        }
        tx.commit().await?;
        invoice_response(pool, business_id, invoice_id).await
    }
}

async fn enqueue_webhook_tx(tx: &mut Transaction<'_, Postgres>, business_id: Uuid, event_type: &str, payload: Value) -> AppResult<()> {
    sqlx::query("INSERT INTO webhook_outbox (business_id, event_type, payload) VALUES ($1, $2, $3)")
        .bind(business_id)
        .bind(event_type)
        .bind(payload)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[async_trait]
impl CustomerRepository for PostgresRepo {
    async fn create_customer(&self, business_id: Uuid, req: CreateCustomerRequest) -> AppResult<CustomerResponse> {
        sqlx::query_as::<_, CustomerResponse>("INSERT INTO customers (business_id, email, name, metadata) VALUES ($1, $2, $3, $4) RETURNING id, email, name, metadata, created_at")
            .bind(business_id)
            .bind(req.email)
            .bind(req.name)
            .bind(req.metadata)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)
    }

    async fn list_customers(&self, business_id: Uuid) -> AppResult<Vec<CustomerResponse>> {
        sqlx::query_as::<_, CustomerResponse>("SELECT id, email, name, metadata, created_at FROM customers WHERE business_id = $1 ORDER BY created_at DESC")
            .bind(business_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)
    }

    async fn get_customer(&self, business_id: Uuid, id: Uuid) -> AppResult<CustomerResponse> {
        sqlx::query_as::<_, CustomerResponse>("SELECT id, email, name, metadata, created_at FROM customers WHERE business_id = $1 AND id = $2")
            .bind(business_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("customer_not_found"))
    }
}

#[async_trait]
impl InvoiceRepository for PostgresRepo {
    async fn create_invoice(&self, business_id: Uuid, req: CreateInvoiceRequest) -> AppResult<InvoiceResponse> {
        if req.line_items.is_empty() {
            return Err(AppError::BadRequest("line_items_required", "At least one line item is required"));
        }
        let total: i32 = req.line_items.iter().map(|i| i.quantity * i.unit_amount_cents).sum();
        let mut tx = self.pool.begin().await?;
        let invoice_id: Uuid = sqlx::query_scalar("INSERT INTO invoices (business_id, customer_id, state, currency, amount_due_cents) VALUES ($1, $2, 'draft', $3, $4) RETURNING id")
            .bind(business_id)
            .bind(req.customer_id)
            .bind(req.currency)
            .bind(total)
            .fetch_one(&mut *tx)
            .await?;
        for item in req.line_items {
            sqlx::query("INSERT INTO line_items (invoice_id, description, quantity, unit_amount_cents) VALUES ($1, $2, $3, $4)")
                .bind(invoice_id)
                .bind(item.description)
                .bind(item.quantity)
                .bind(item.unit_amount_cents)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        invoice_response(&self.pool, business_id, invoice_id).await
    }

    async fn list_invoices(&self, business_id: Uuid) -> AppResult<Vec<InvoiceResponse>> {
        let ids = sqlx::query_scalar::<_, Uuid>("SELECT id FROM invoices WHERE business_id = $1 ORDER BY created_at DESC")
            .bind(business_id)
            .fetch_all(&self.pool)
            .await?;
        let mut invoices = Vec::with_capacity(ids.len());
        for id in ids {
            invoices.push(invoice_response(&self.pool, business_id, id).await?);
        }
        Ok(invoices)
    }

    async fn get_invoice(&self, business_id: Uuid, id: Uuid) -> AppResult<InvoiceResponse> {
        invoice_response(&self.pool, business_id, id).await
    }
}

#[async_trait]
impl PaymentRepository for PostgresRepo {
    async fn pay_invoice_tx(&self, business_id: Uuid, invoice_id: Uuid, req: PayInvoiceRequest, idem_key: String) -> AppResult<(u16, Value)> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT id, state, currency, amount_due_cents, amount_paid_cents FROM invoices WHERE business_id = $1 AND id = $2 FOR UPDATE")
            .bind(business_id)
            .bind(invoice_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound("invoice_not_found"))?;
        let state: String = row.try_get("state")?;
        if state != "open" {
            return Err(AppError::Conflict("invoice_not_payable", "Invoice is not open"));
        }
        let amount_due: i32 = row.try_get("amount_due_cents")?;
        let amount_paid: i32 = row.try_get("amount_paid_cents")?;
        let charge_amount = amount_due - amount_paid;
        if charge_amount <= 0 {
            return Err(AppError::Conflict("invoice_not_payable", "Invoice is already paid"));
        }
        let attempt_id: Uuid = sqlx::query_scalar("INSERT INTO payment_attempts (invoice_id, idempotency_key, token, amount_cents, status) VALUES ($1, $2, $3, $4, 'processing') RETURNING id")
            .bind(invoice_id)
            .bind(&idem_key)
            .bind(&req.payment_token)
            .bind(charge_amount)
            .fetch_one(&mut *tx)
            .await?;

        let currency: String = row.try_get("currency")?;
        let settings = crate::config::settings::Settings::from_env().map_err(|e| AppError::Internal(e.to_string()))?;
        let http = reqwest::Client::new();
        let result = charge(&settings.mock_psp_url, settings.payment_timeout_seconds, &http, ChargeRequest {
            amount_cents: charge_amount,
            currency: &currency,
            token: &req.payment_token,
            idempotency_key: &idem_key,
        }).await;

        match result {
            Ok(psp) if psp.status == "succeeded" => {
                sqlx::query("UPDATE payment_attempts SET status = 'succeeded', psp_ref = $1, updated_at = now() WHERE id = $2")
                    .bind(psp.psp_ref)
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("UPDATE invoices SET state = 'paid', amount_paid_cents = amount_due_cents, updated_at = now() WHERE id = $1")
                    .bind(invoice_id)
                    .execute(&mut *tx)
                    .await?;
                enqueue_webhook_tx(&mut tx, business_id, "invoice.paid", json!({"invoice_id": invoice_id, "state": "paid"})).await?;
                tx.commit().await?;
                Ok((200, json!({"status":"succeeded","invoice_id":invoice_id})))
            }
            Ok(psp) => {
                sqlx::query("UPDATE payment_attempts SET status = 'failed', failure_code = $1, updated_at = now() WHERE id = $2")
                    .bind(psp.code.unwrap_or_else(|| "payment_failed".to_string()))
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                enqueue_webhook_tx(&mut tx, business_id, "invoice.payment_failed", json!({"invoice_id": invoice_id, "state": "open"})).await?;
                tx.commit().await?;
                Ok((402, json!({"status":"failed"})))
            }
            Err(e) if e.is_timeout() => {
                sqlx::query("UPDATE payment_attempts SET status = 'processing', updated_at = now() WHERE id = $1")
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                Ok((202, json!({"status":"processing"})))
            }
            Err(_) => {
                sqlx::query("UPDATE payment_attempts SET status = 'unknown', failure_code = 'network_error', updated_at = now() WHERE id = $1")
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                Ok((202, json!({"status":"processing"})))
            }
        }
    }
}

#[async_trait]
impl WebhookRepository for PostgresRepo {
    async fn create_webhook(&self, business_id: Uuid, req: CreateWebhookRequest) -> AppResult<WebhookResponse> {
        let secret: String = rand::thread_rng().sample_iter(&Alphanumeric).take(48).map(char::from).collect();
        sqlx::query_as::<_, WebhookResponse>("INSERT INTO webhook_endpoints (business_id, url, secret) VALUES ($1, $2, $3) RETURNING id, url, enabled, created_at")
            .bind(business_id)
            .bind(req.url)
            .bind(secret)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)
    }

    async fn list_webhook_events(&self, business_id: Uuid) -> AppResult<Vec<Value>> {
        let rows = sqlx::query("SELECT id, event_type, payload, attempts, next_attempt_at, delivered_at FROM webhook_outbox WHERE business_id = $1 ORDER BY created_at DESC")
            .bind(business_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| json!({
            "id": r.get::<Uuid,_>("id"),
            "event_type": r.get::<String,_>("event_type"),
            "payload": r.get::<Value,_>("payload"),
            "attempts": r.get::<i32,_>("attempts"),
            "next_attempt_at": r.get::<chrono::DateTime<Utc>,_>("next_attempt_at"),
            "delivered_at": r.get::<Option<chrono::DateTime<Utc>>,_>("delivered_at")
        })).collect())
    }
}

#[async_trait]
impl ApiKeyRepository for PostgresRepo {
    async fn authenticate(&self, key: &str) -> AppResult<Uuid> {
        let rows = sqlx::query("SELECT business_id, key_hash FROM api_keys WHERE revoked_at IS NULL")
            .fetch_all(&self.pool)
            .await?;
        for row in rows {
            let hash: String = row.try_get("key_hash")?;
            if verify_api_key(key, &hash) {
                return Ok(row.try_get("business_id")?);
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn create_api_key(&self, business_id: Uuid) -> AppResult<String> {
        let suffix: String = rand::thread_rng().sample_iter(&Alphanumeric).take(43).map(char::from).collect();
        let key = format!("dodo_live_{}", suffix);
        let hash = hash_api_key(&key).map_err(|e| AppError::Internal(e.to_string()))?;
        sqlx::query("INSERT INTO api_keys (business_id, key_prefix, key_hash) VALUES ($1, $2, $3)")
            .bind(business_id)
            .bind("dodo_live")
            .bind(hash)
            .execute(&self.pool)
            .await?;
        Ok(key)
    }
}

#[async_trait]
impl IdempotencyRepository for PostgresRepo {
    async fn get_idempotency(&self, business_id: Uuid, key: &str) -> AppResult<Option<(String, i32, Value)>> {
        let row = sqlx::query("SELECT request_hash, response_status, response_body FROM idempotency_keys WHERE business_id = $1 AND key = $2 AND expires_at > now()")
            .bind(business_id)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| (r.get("request_hash"), r.get("response_status"), r.get("response_body"))))
    }

    async fn save_idempotency(&self, business_id: Uuid, key: &str, request_hash: &str, status: u16, response: &Value) -> AppResult<()> {
        sqlx::query("INSERT INTO idempotency_keys (business_id, key, request_hash, response_status, response_body, expires_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(business_id)
            .bind(key)
            .bind(request_hash)
            .bind(status as i32)
            .bind(response)
            .bind(Utc::now() + Duration::hours(24))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
