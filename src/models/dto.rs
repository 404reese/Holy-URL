use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortenRequest {
    pub long_url: String,
    // TODO: validate non-empty and URL format at handler/service layer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrlResponse {
    pub short_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
}
