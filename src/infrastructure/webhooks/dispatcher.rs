use crate::{domain::services::webhook_service::sign, infrastructure::repositories::WebhookDelivery};
use chrono::Utc;
use reqwest::Client;

pub async fn dispatch(client: &Client, delivery: &WebhookDelivery) -> Result<(), reqwest::Error> {
    let timestamp = Utc::now().timestamp();
    let body = delivery.payload.to_string();
    let signature = sign(&delivery.secret, timestamp, &body);
    client
        .post(&delivery.url)
        .header("X-Dodo-Signature", signature)
        .header("X-Dodo-Timestamp", timestamp.to_string())
        .header("X-Dodo-Event-Id", delivery.event_id.to_string())
        .body(body)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
