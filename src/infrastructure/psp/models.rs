use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChargeRequest<'a> {
    pub amount_cents: i32,
    pub currency: &'a str,
    pub token: &'a str,
    pub idempotency_key: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ChargeResponse {
    pub status: String,
    pub psp_ref: Option<String>,
    pub code: Option<String>,
}
