use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub business_id: Uuid,
    pub email: String,
    pub name: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
