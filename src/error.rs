//! Error types for the event indexer

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad request"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        
        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
        }));
        
        (status, body).into_response()
    }
}
