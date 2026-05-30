use crate::{application::{dto::PayInvoiceRequest, AppState}, domain::repositories::PaymentRepository, shared::errors::AppResult};
use serde_json::Value;
use uuid::Uuid;

pub async fn execute(state: &AppState, business_id: Uuid, invoice_id: Uuid, req: PayInvoiceRequest, idem_key: String) -> AppResult<(u16, Value)> {
    state.repo.pay_invoice_tx(business_id, invoice_id, req, idem_key).await
}
