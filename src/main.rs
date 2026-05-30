mod config;
mod domain;
mod application;
mod infrastructure;
mod interfaces;
mod shared;

use crate::config::settings::Settings;
use crate::infrastructure::db::postgres::{migrate, seed_business_and_key};
use crate::infrastructure::repositories::PostgresRepo;
use crate::infrastructure::workers::{spawn_idempotency_cleanup_worker, spawn_payment_recovery_worker, spawn_webhook_worker};
use crate::interfaces::http::routes::router;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let settings = Settings::from_env()?;
    let pool = PgPoolOptions::new().max_connections(20).connect(&settings.database_url).await?;
    migrate(&pool).await?;
    seed_business_and_key(&pool, &settings.seed_api_key).await?;

    let repo = Arc::new(PostgresRepo::new(pool.clone()));
    let app_state = Arc::new(application::AppState::new(settings.clone(), repo));

    spawn_payment_recovery_worker(app_state.clone());
    spawn_webhook_worker(app_state.clone());
    spawn_idempotency_cleanup_worker(pool);

    let app = router(app_state).layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(&settings.bind_addr).await?;
    tracing::info!(addr = %settings.bind_addr, "invoice service listening");
    axum::serve(listener, app).await?;
    Ok(())
}
