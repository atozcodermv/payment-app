use crate::{application::dto::{CreateInvoiceRequest, InvoiceResponse}, shared::errors::AppResult};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn create_invoice(&self, business_id: Uuid, req: CreateInvoiceRequest) -> AppResult<InvoiceResponse>;
    async fn list_invoices(&self, business_id: Uuid) -> AppResult<Vec<InvoiceResponse>>;
    async fn get_invoice(&self, business_id: Uuid, id: Uuid) -> AppResult<InvoiceResponse>;
}
