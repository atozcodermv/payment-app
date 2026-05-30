use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub business_id: Uuid,
    pub key_prefix: String,
    pub revoked_at: Option<DateTime<Utc>>,
}
