use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub business_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateCustomerRequest {
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateLineItemRequest {
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i32,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateInvoiceRequest {
    pub customer_id: Uuid,
    pub currency: String,
    pub line_items: Vec<CreateLineItemRequest>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct LineItemResponse {
    pub id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i32,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub state: String,
    pub currency: String,
    pub amount_due_cents: i32,
    pub amount_paid_cents: i32,
    pub line_items: Vec<LineItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct PayInvoiceRequest {
    pub payment_token: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}
