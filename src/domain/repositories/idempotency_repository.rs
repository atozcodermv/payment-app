use crate::shared::errors::AppResult;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    async fn get_idempotency(&self, business_id: Uuid, key: &str) -> AppResult<Option<(String, i32, Value)>>;
    async fn save_idempotency(&self, business_id: Uuid, key: &str, request_hash: &str, status: u16, response: &Value) -> AppResult<()>;
}
