use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Pool};
use tracing::instrument;

use crate::errors::{AppError, AppResult};

#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn get(&self, short_id: &str) -> AppResult<Option<String>>;
    async fn set(&self, short_id: &str, long_url: &str, ttl_seconds: usize) -> AppResult<()>;
}

pub struct RedisRepo {
    pub pool: Pool,
}

impl RedisRepo {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CacheRepository for RedisRepo {
    #[instrument(skip(self), fields(short_id = %short_id))]
    async fn get(&self, short_id: &str) -> AppResult<Option<String>> {
        let mut conn = self.pool.get().await.map_err(map_redis_err)?;
        let result: Option<String> = conn.get(short_id).await.map_err(map_redis_err)?;
        Ok(result)
    }

    #[instrument(skip(self, long_url), fields(short_id = %short_id, ttl = ttl_seconds))]
    async fn set(&self, short_id: &str, long_url: &str, ttl_seconds: usize) -> AppResult<()> {
        let mut conn = self.pool.get().await.map_err(map_redis_err)?;
        conn.set_ex(short_id, long_url, ttl_seconds)
            .await
            .map_err(map_redis_err)?;
        Ok(())
    }
}

fn map_redis_err<E: ToString>(err: E) -> AppError {
    AppError::CacheError(err.to_string())
}

// TODO: configure deadpool-redis pool (url, auth, size) at composition root.
