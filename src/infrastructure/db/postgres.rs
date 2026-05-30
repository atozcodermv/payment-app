use crate::{infrastructure::auth::argon::hash_api_key, shared::constants::DEFAULT_BUSINESS_NAME};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn seed_business_and_key(pool: &PgPool, seed_api_key: &str) -> anyhow::Result<Uuid> {
    let existing = sqlx::query("SELECT id FROM businesses LIMIT 1").fetch_optional(pool).await?;
    let business_id = if let Some(row) = existing {
        row.try_get("id")?
    } else {
        sqlx::query_scalar("INSERT INTO businesses (name) VALUES ($1) RETURNING id")
            .bind(DEFAULT_BUSINESS_NAME)
            .fetch_one(pool)
            .await?
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE business_id = $1")
        .bind(business_id)
        .fetch_one(pool)
        .await?;
    if count == 0 {
        let hash = hash_api_key(seed_api_key)?;
        sqlx::query("INSERT INTO api_keys (business_id, key_prefix, key_hash) VALUES ($1, $2, $3)")
            .bind(business_id)
            .bind("dodo_live")
            .bind(hash)
            .execute(pool)
            .await?;
        tracing::info!("seeded development API key from SEED_API_KEY");
    }
    Ok(business_id)
}
