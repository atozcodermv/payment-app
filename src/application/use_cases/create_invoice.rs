use crate::{application::{dto::{CreateInvoiceRequest, InvoiceResponse}, AppState}, domain::repositories::InvoiceRepository, shared::errors::AppResult};
use uuid::Uuid;

pub async fn execute(state: &AppState, business_id: Uuid, req: CreateInvoiceRequest) -> AppResult<InvoiceResponse> {
    state.repo.create_invoice(business_id, req).await
}
