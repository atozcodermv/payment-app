use crate::{application::AppState, infrastructure::{psp::{client::charge, models::ChargeRequest}, webhooks::{dispatcher::dispatch, retry_worker::retry_delay_minutes}}};
use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

pub fn spawn_payment_recovery_worker(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = recover_payments(state.clone()).await {
                tracing::warn!(error = %err, "payment recovery failed");
            }
            sleep(TokioDuration::from_secs(60)).await;
        }
    });
}

async fn recover_payments(state: Arc<AppState>) -> anyhow::Result<()> {
    let attempts = sqlx::query("SELECT pa.id, pa.invoice_id, pa.idempotency_key, pa.token, pa.amount_cents, i.business_id, i.currency FROM payment_attempts pa JOIN invoices i ON i.id = pa.invoice_id WHERE pa.status IN ('processing','unknown') AND pa.updated_at < now() - interval '5 minutes' LIMIT 50")
        .fetch_all(&state.repo.pool)
        .await?;
    for row in attempts {
        let attempt_id: Uuid = row.get("id");
        let invoice_id: Uuid = row.get("invoice_id");
        let business_id: Uuid = row.get("business_id");
        let idem: String = row.get("idempotency_key");
        let token: String = row.get("token");
        let amount: i32 = row.get("amount_cents");
        let currency: String = row.get("currency");
        let psp = charge(&state.settings.mock_psp_url, state.settings.payment_timeout_seconds, &state.http, ChargeRequest {
            amount_cents: amount,
            currency: &currency,
            token: &token,
            idempotency_key: &idem,
        }).await;
        let mut tx = state.repo.pool.begin().await?;
        if let Ok(resp) = psp {
            if resp.status == "succeeded" {
                sqlx::query("UPDATE payment_attempts SET status = 'succeeded', psp_ref = $1, updated_at = now() WHERE id = $2")
                    .bind(resp.psp_ref)
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("UPDATE invoices SET state = 'paid', amount_paid_cents = amount_due_cents, updated_at = now() WHERE id = $1 AND state = 'open'")
                    .bind(invoice_id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("INSERT INTO webhook_outbox (business_id, event_type, payload) VALUES ($1, 'invoice.paid', $2)")
                    .bind(business_id)
                    .bind(serde_json::json!({"invoice_id": invoice_id, "state": "paid"}))
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query("UPDATE payment_attempts SET status = 'failed', failure_code = $1, updated_at = now() WHERE id = $2")
                    .bind(resp.code.unwrap_or_else(|| "payment_failed".to_string()))
                    .bind(attempt_id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("INSERT INTO webhook_outbox (business_id, event_type, payload) VALUES ($1, 'invoice.payment_failed', $2)")
                    .bind(business_id)
                    .bind(serde_json::json!({"invoice_id": invoice_id, "state": "open"}))
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
    }
    Ok(())
}

pub fn spawn_webhook_worker(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = deliver_webhooks(state.clone()).await {
                tracing::warn!(error = %err, "webhook worker failed");
            }
            sleep(TokioDuration::from_secs(10)).await;
        }
    });
}

async fn deliver_webhooks(state: Arc<AppState>) -> anyhow::Result<()> {
    let mut tx = state.repo.pool.begin().await?;
    let rows = sqlx::query("SELECT id, business_id, event_type, payload, attempts FROM webhook_outbox WHERE delivered_at IS NULL AND next_attempt_at <= now() ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 20")
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;
    for row in rows {
        let event_id: Uuid = row.get("id");
        let business_id: Uuid = row.get("business_id");
        let payload: Value = row.get("payload");
        let attempts: i32 = row.get("attempts");
        let endpoints = sqlx::query("SELECT url, secret FROM webhook_endpoints WHERE business_id = $1 AND enabled = true")
            .bind(business_id)
            .fetch_all(&state.repo.pool)
            .await?;
        let mut ok = !endpoints.is_empty();
        for ep in endpoints {
            let delivery = crate::infrastructure::repositories::WebhookDelivery {
                event_id,
                url: ep.get("url"),
                secret: ep.get("secret"),
                payload: payload.clone(),
            };
            ok = ok && dispatch(&state.http, &delivery).await.is_ok();
        }
        if ok {
            sqlx::query("UPDATE webhook_outbox SET delivered_at = now() WHERE id = $1")
                .bind(event_id)
                .execute(&state.repo.pool)
                .await?;
        } else if let Some(minutes) = retry_delay_minutes(attempts) {
            sqlx::query("UPDATE webhook_outbox SET attempts = attempts + 1, next_attempt_at = $1 WHERE id = $2")
                .bind(Utc::now() + Duration::minutes(minutes))
                .bind(event_id)
                .execute(&state.repo.pool)
                .await?;
        } else {
            sqlx::query("INSERT INTO webhook_dead_letters (outbox_id, business_id, event_type, payload, attempts, last_error) SELECT id, business_id, event_type, payload, attempts, 'retry_exhausted' FROM webhook_outbox WHERE id = $1")
                .bind(event_id)
                .execute(&state.repo.pool)
                .await?;
            sqlx::query("UPDATE webhook_outbox SET delivered_at = now() WHERE id = $1")
                .bind(event_id)
                .execute(&state.repo.pool)
                .await?;
        }
    }
    Ok(())
}

pub fn spawn_idempotency_cleanup_worker(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = sqlx::query("DELETE FROM idempotency_keys WHERE expires_at <= now()").execute(&pool).await {
                tracing::warn!(error = %err, "idempotency cleanup failed");
            }
            sleep(TokioDuration::from_secs(3600)).await;
        }
    });
}
