use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn authenticate(&self, key: &str) -> AppResult<Uuid>;
    async fn create_api_key(&self, business_id: Uuid) -> AppResult<String>;
}
