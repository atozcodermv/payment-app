use crate::{
    application::{dto::*, use_cases, AppState},
    domain::repositories::{ApiKeyRepository, CustomerRepository, IdempotencyRepository, InvoiceRepository, WebhookRepository},
    infrastructure::auth::api_key_middleware::AuthBusiness,
    shared::{errors::{AppError, AppResult}, response::ListResponse, utils::request_hash},
};
use axum::{body::Bytes, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, Json};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok"}))
}

pub async fn create_api_key(State(state): State<Arc<AppState>>, Json(req): Json<CreateApiKeyRequest>) -> AppResult<Json<CreateApiKeyResponse>> {
    let business_id = if let Some(id) = req.business_id {
        id
    } else {
        sqlx::query_scalar("SELECT id FROM businesses LIMIT 1")
            .fetch_one(&state.repo.pool)
            .await?
    };
    Ok(Json(CreateApiKeyResponse { api_key: state.repo.create_api_key(business_id).await? }))
}

pub async fn create_customer(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Json(req): Json<CreateCustomerRequest>) -> AppResult<Json<CustomerResponse>> {
    Ok(Json(use_cases::create_customer::execute(&state, business_id, req).await?))
}

pub async fn list_customers(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>) -> AppResult<Json<ListResponse<CustomerResponse>>> {
    Ok(Json(ListResponse { data: state.repo.list_customers(business_id).await? }))
}

pub async fn get_customer(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<CustomerResponse>> {
    Ok(Json(state.repo.get_customer(business_id, id).await?))
}

pub async fn create_invoice(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Json(req): Json<CreateInvoiceRequest>) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(use_cases::create_invoice::execute(&state, business_id, req).await?))
}

pub async fn list_invoices(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>) -> AppResult<Json<ListResponse<InvoiceResponse>>> {
    Ok(Json(ListResponse { data: state.repo.list_invoices(business_id).await? }))
}

pub async fn get_invoice(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(state.repo.get_invoice(business_id, id).await?))
}

pub async fn finalize_invoice(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(use_cases::finalize_invoice::execute(&state, business_id, id).await?))
}

pub async fn void_invoice(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(use_cases::void_invoice::execute(&state, business_id, id).await?))
}

pub async fn writeoff_invoice(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(use_cases::writeoff_invoice::execute(&state, business_id, id).await?))
}

pub async fn pay_invoice(
    AuthBusiness(business_id): AuthBusiness,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Response> {
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("idempotency_key_required", "Idempotency-Key header is required"))?;
    let hash = request_hash(&body);
    if let Some((stored_hash, status, cached_body)) = state.repo.get_idempotency(business_id, key).await? {
        if stored_hash != hash {
            return Err(AppError::BadRequest("idempotency_key_reused", "Idempotency key was reused with a different request"));
        }
        return Ok((StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK), Json(cached_body)).into_response());
    }
    let req: PayInvoiceRequest = serde_json::from_slice(&body).map_err(|_| AppError::BadRequest("invalid_json", "Request body is invalid JSON"))?;
    let (status, response_body) = use_cases::pay_invoice::execute(&state, business_id, id, req, key.to_string()).await?;
    state.repo.save_idempotency(business_id, key, &hash, status, &response_body).await?;
    Ok((StatusCode::from_u16(status).unwrap_or(StatusCode::OK), Json(response_body)).into_response())
}

pub async fn create_webhook(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>, Json(req): Json<CreateWebhookRequest>) -> AppResult<Json<WebhookResponse>> {
    Ok(Json(use_cases::register_webhook::execute(&state, business_id, req).await?))
}

pub async fn list_webhook_events(AuthBusiness(business_id): AuthBusiness, State(state): State<Arc<AppState>>) -> AppResult<Json<ListResponse<Value>>> {
    Ok(Json(ListResponse { data: state.repo.list_webhook_events(business_id).await? }))
}
