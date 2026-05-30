use crate::{application::{dto::{CreateCustomerRequest, CustomerResponse}, AppState}, domain::repositories::CustomerRepository, shared::errors::AppResult};
use uuid::Uuid;

pub async fn execute(state: &AppState, business_id: Uuid, req: CreateCustomerRequest) -> AppResult<CustomerResponse> {
    state.repo.create_customer(business_id, req).await
}
