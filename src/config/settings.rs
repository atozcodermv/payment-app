use std::env;

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: String,
    pub bind_addr: String,
    pub mock_psp_url: String,
    pub payment_timeout_seconds: u64,
    pub webhook_secret: String,
    pub seed_api_key: String,
}

impl Settings {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            mock_psp_url: env::var("MOCK_PSP_URL").unwrap_or_else(|_| "http://localhost:8081".to_string()),
            payment_timeout_seconds: env::var("PAYMENT_TIMEOUT_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(5),
            webhook_secret: env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "dev_webhook_secret_change_me".to_string()),
            seed_api_key: env::var("SEED_API_KEY").unwrap_or_else(|_| "dodo_live_dev_secret_key".to_string()),
        })
    }
}
