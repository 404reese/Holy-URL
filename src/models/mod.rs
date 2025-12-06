use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod dto;

pub use dto::{ErrorResponse, HealthResponse, ShortUrlResponse, ShortenRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEntry {
    pub short_id: String,
    pub long_url: String,
    pub created_at: DateTime<Utc>,
}
