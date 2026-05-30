use crate::{application::dto::PayInvoiceRequest, shared::errors::AppResult};
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn pay_invoice_tx(&self, business_id: Uuid, invoice_id: Uuid, req: PayInvoiceRequest, idem_key: String) -> AppResult<(u16, Value)>;
}
