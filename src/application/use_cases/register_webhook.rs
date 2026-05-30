use crate::{application::{dto::{CreateWebhookRequest, WebhookResponse}, AppState}, domain::repositories::WebhookRepository, shared::errors::AppResult};
use uuid::Uuid;

pub async fn execute(state: &AppState, business_id: Uuid, req: CreateWebhookRequest) -> AppResult<WebhookResponse> {
    state.repo.create_webhook(business_id, req).await
}
