use crate::{application::AppState, interfaces::{http::handlers, openapi::docs::ApiDoc}};
use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api-keys", post(handlers::create_api_key))
        .route("/customers", post(handlers::create_customer).get(handlers::list_customers))
        .route("/customers/:id", get(handlers::get_customer))
        .route("/invoices", post(handlers::create_invoice).get(handlers::list_invoices))
        .route("/invoices/:id", get(handlers::get_invoice))
        .route("/invoices/:id/finalize", post(handlers::finalize_invoice))
        .route("/invoices/:id/void", post(handlers::void_invoice))
        .route("/invoices/:id/writeoff", post(handlers::writeoff_invoice))
        .route("/invoices/:id/pay", post(handlers::pay_invoice))
        .route("/webhooks", post(handlers::create_webhook))
        .route("/webhooks/events", get(handlers::list_webhook_events))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
