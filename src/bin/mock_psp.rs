use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, time::{sleep, Duration}};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ChargeRequest {
    token: String,
    amount_cents: i32,
    currency: String,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
struct ChargeResponse {
    status: String,
    psp_ref: Option<String>,
    code: Option<String>,
}

async fn charge(Json(req): Json<ChargeRequest>) -> (axum::http::StatusCode, Json<ChargeResponse>) {
    let _ = (req.amount_cents, req.currency, req.idempotency_key);
    match req.token.as_str() {
        "tok_success" => {
            sleep(Duration::from_millis(100)).await;
            (axum::http::StatusCode::OK, Json(ChargeResponse { status: "succeeded".into(), psp_ref: Some(Uuid::new_v4().to_string()), code: None }))
        }
        "tok_insufficient_funds" => (axum::http::StatusCode::OK, Json(ChargeResponse { status: "failed".into(), psp_ref: None, code: Some("insufficient_funds".into()) })),
        "tok_card_declined" => (axum::http::StatusCode::OK, Json(ChargeResponse { status: "failed".into(), psp_ref: None, code: Some("card_declined".into()) })),
        "tok_timeout" => {
            sleep(Duration::from_secs(30)).await;
            (axum::http::StatusCode::OK, Json(ChargeResponse { status: "succeeded".into(), psp_ref: Some(Uuid::new_v4().to_string()), code: None }))
        }
        "tok_network_error" => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ChargeResponse { status: "failed".into(), psp_ref: None, code: Some("network_error".into()) })),
        _ => (axum::http::StatusCode::OK, Json(ChargeResponse { status: "failed".into(), psp_ref: None, code: Some("invalid_token".into()) })),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!(addr = %bind_addr, "mock psp listening");
    axum::serve(listener, Router::new().route("/charge", post(charge))).await?;
    Ok(())
}
