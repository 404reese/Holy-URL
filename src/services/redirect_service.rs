use std::sync::Arc;

use async_trait::async_trait;
use tracing::{instrument, warn};

use crate::{
    errors::{AppError, AppResult},
    repositories::{CacheRepository, UrlRepository},
};

const CACHE_TTL_SECONDS: u64 = 60 * 60 * 24; // 24h

#[async_trait]
pub trait RedirectService: Send + Sync {
    async fn resolve(&self, short_id: &str) -> AppResult<String>;
}

pub struct RedirectServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    cache: Arc<R>,
    store: Arc<D>,
}

impl<R, D> RedirectServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    pub fn new(cache: Arc<R>, store: Arc<D>) -> Self {
        Self { cache, store }
    }

    #[instrument(skip(self), fields(short_id = %short_id))]
    pub async fn resolve_url(&self, short_id: &str) -> AppResult<String> {
        if short_id.is_empty() {
            return Err(AppError::NotFound);
        }

        if let Some(long_url) = self.cache.get(short_id).await? {
            return Ok(long_url);
        }

        let entry = self.store.find(short_id).await?; // TODO: Cassandra read implementation
        if let Some(found) = entry {
            if let Err(err) = self
                .cache
                .set(short_id, &found, CACHE_TTL_SECONDS)
                .await
            {
                warn!(error = %err, "failed to populate redis cache"); // TODO: Redis cache implementation
            }
            return Ok(found);
        }

        Err(AppError::NotFound)
    }
}

#[async_trait]
impl<R, D> RedirectService for RedirectServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    #[instrument(skip(self), fields(short_id = %short_id))]
    async fn resolve(&self, short_id: &str) -> AppResult<String> {
        self.resolve_url(short_id).await
    }
}
