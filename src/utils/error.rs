use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{error::Error, fmt};
use tracing::error;

#[derive(Debug, Clone)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    DatabaseError(String),
    CacheError(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "bad request: {}", msg),
            AppError::NotFound(msg) => write!(f, "not found: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "database error: {}", msg),
            AppError::CacheError(msg) => write!(f, "cache error: {}", msg),
            AppError::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::CacheError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        error!(status = status.as_u16(), message = %message, "request failed");

        let payload = json!({
            "message": message,
            "code": status.as_u16(),
        });

        let body = serde_json::to_string(&payload)
            .unwrap_or_else(|_| "{\"message\":\"internal error\",\"code\":500}".to_string());

        let mut response = Response::new(Body::from(body));
        *response.status_mut() = status;
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
    }
}
