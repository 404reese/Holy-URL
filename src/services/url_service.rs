use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{ShortUrlResponse, UrlEntry},
    repositories::{CacheRepository, UrlRepository},
};

const BASE62_ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const MAX_ID_ATTEMPTS: usize = 5;

#[async_trait]
pub trait UrlService: Send + Sync {
    async fn shorten(&self, long_url: String) -> AppResult<ShortUrlResponse>;
    async fn generate_id(&self) -> String;
}

pub struct UrlServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    base_url: String,
    cache: Arc<R>,
    store: Arc<D>,
}

impl<R, D> UrlServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    pub fn new(base_url: impl Into<String>, cache: Arc<R>, store: Arc<D>) -> Self {
        let normalized_base = normalize_base_url(base_url.into());
        Self {
            base_url: normalized_base,
            cache,
            store,
        }
    }

    #[instrument(skip(self, long_url), fields(long_url = %long_url))]
    pub async fn shorten_url(&self, long_url: String) -> AppResult<ShortUrlResponse> {
        let parsed = Url::parse(&long_url)
            .map_err(|_| AppError::InvalidUrl(long_url.clone()))?;
        if !is_supported_scheme(&parsed) {
            return Err(AppError::InvalidUrl(long_url));
        }

        let short_id = self.generate_unique_id().await?;
        let created_at = Utc::now();
        let entry = UrlEntry {
            short_id: short_id.clone(),
            long_url: parsed.to_string(),
            created_at,
        };

        // Persist first to maintain source of truth; caching is best-effort for read optimization.
        self.store.save(entry).await?; // TODO: wire Cassandra storage implementation
        self.cache
            .set(&short_id, &long_url)
            .await?; // TODO: wire Redis caching implementation

        Ok(ShortUrlResponse {
            short_url: format!("{}/{}", self.base_url, short_id),
        })
    }

    #[instrument(skip(self))]
    async fn generate_unique_id(&self) -> AppResult<String> {
        for _ in 0..MAX_ID_ATTEMPTS {
            let candidate = self.generate_id().await;
            let exists = self.store.exists(&candidate).await?; // TODO: Cassandra existence check
            if !exists {
                return Ok(candidate);
            }
        }
        Err(AppError::Unexpected(
            "exhausted attempts generating unique short id".to_string(),
        ))
    }

    fn base62_encode(mut num: u128) -> String {
        if num == 0 {
            return "0".to_string();
        }

        let mut buf = Vec::with_capacity(22); // fits encoded u128
        while num > 0 {
            let rem = (num % 62) as usize;
            buf.push(BASE62_ALPHABET[rem]);
            num /= 62;
        }
        buf.reverse();
        // SAFETY: BASE62_ALPHABET is ASCII-only
        String::from_utf8(buf).expect("base62 encoding produced invalid utf8")
    }
}

#[async_trait]
impl<R, D> UrlService for UrlServiceImpl<R, D>
where
    R: CacheRepository,
    D: UrlRepository,
{
    #[instrument(skip(self, long_url), fields(long_url = %long_url))]
    async fn shorten(&self, long_url: String) -> AppResult<ShortUrlResponse> {
        self.shorten_url(long_url).await
    }

    #[instrument(skip(self))]
    async fn generate_id(&self) -> String {
        let uuid = Uuid::new_v4();
        Self::base62_encode(uuid.as_u128())
    }
}

fn normalize_base_url(base: String) -> String {
    base.trim_end_matches('/').to_owned()
}

fn is_supported_scheme(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}
