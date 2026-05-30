use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Invoice {
    pub id: Uuid,
    pub business_id: Uuid,
    pub customer_id: Uuid,
    pub state: String,
    pub currency: String,
    pub amount_due_cents: i32,
    pub amount_paid_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub line_items: Vec<LineItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WebhookPayload {
    pub invoice_id: Uuid,
    pub state: String,
    pub data: Value,
}
