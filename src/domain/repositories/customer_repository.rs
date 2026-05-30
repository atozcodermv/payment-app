use crate::{application::dto::{CreateCustomerRequest, CustomerResponse}, shared::errors::AppResult};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create_customer(&self, business_id: Uuid, req: CreateCustomerRequest) -> AppResult<CustomerResponse>;
    async fn list_customers(&self, business_id: Uuid) -> AppResult<Vec<CustomerResponse>>;
    async fn get_customer(&self, business_id: Uuid, id: Uuid) -> AppResult<CustomerResponse>;
}
