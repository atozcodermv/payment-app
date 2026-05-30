pub mod commands;
pub mod dto;
pub mod queries;
pub mod use_cases;

use crate::{config::settings::Settings, infrastructure::repositories::PostgresRepo};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub repo: Arc<PostgresRepo>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(settings: Settings, repo: Arc<PostgresRepo>) -> Self {
        Self { settings, repo, http: reqwest::Client::new() }
    }
}
