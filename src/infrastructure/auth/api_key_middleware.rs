use crate::{application::AppState, domain::repositories::ApiKeyRepository, shared::errors::AppError};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct AuthBusiness(pub Uuid);

impl FromRequestParts<Arc<AppState>> for AuthBusiness {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .or_else(|| {
                parts.headers.get("authorization").and_then(|v| {
                    let s = v.to_str().ok()?;
                    s.strip_prefix("Bearer ")
                })
            })
            .ok_or(AppError::Unauthorized)?;
        Ok(Self(state.repo.authenticate(key).await?))
    }
}
