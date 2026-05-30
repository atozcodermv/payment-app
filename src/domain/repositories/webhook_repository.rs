use crate::{application::dto::{CreateWebhookRequest, WebhookResponse}, shared::errors::AppResult};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create_webhook(&self, business_id: Uuid, req: CreateWebhookRequest) -> AppResult<WebhookResponse>;
    async fn list_webhook_events(&self, business_id: Uuid) -> AppResult<Vec<serde_json::Value>>;
}
