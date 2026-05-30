use crate::{application::{dto::InvoiceResponse, AppState}, infrastructure::repositories::invoice_transitions, shared::errors::AppResult};
use uuid::Uuid;

pub async fn execute(state: &AppState, business_id: Uuid, invoice_id: Uuid) -> AppResult<InvoiceResponse> {
    invoice_transitions::transition(&state.repo.pool, business_id, invoice_id, "open").await
}
