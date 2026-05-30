use crate::infrastructure::psp::models::{ChargeRequest, ChargeResponse};
use reqwest::Client;
use std::time::Duration;

pub async fn charge(base_url: &str, timeout_secs: u64, client: &Client, req: ChargeRequest<'_>) -> Result<ChargeResponse, reqwest::Error> {
    client
        .post(format!("{}/charge", base_url.trim_end_matches('/')))
        .timeout(Duration::from_secs(timeout_secs))
        .json(&req)
        .send()
        .await?
        .error_for_status()?
        .json::<ChargeResponse>()
        .await
}
