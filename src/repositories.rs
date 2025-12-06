use async_trait::async_trait;

use crate::{errors::AppResult, models::UrlEntry};

#[async_trait]
pub trait UrlRepository: Send + Sync {
    async fn save(&self, entry: UrlEntry) -> AppResult<()>;
    async fn exists(&self, short_id: &str) -> AppResult<bool>;
    async fn get(&self, short_id: &str) -> AppResult<Option<UrlEntry>>;
    // TODO: implement Cassandra persistence layer
}

#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn set(&self, short_id: &str, long_url: &str) -> AppResult<()>;
    async fn get(&self, short_id: &str) -> AppResult<Option<String>>;
    async fn set_with_ttl(
        &self,
        short_id: &str,
        long_url: &str,
        ttl_seconds: u64,
    ) -> AppResult<()>;
    // TODO: implement Redis caching layer
}
