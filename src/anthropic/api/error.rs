use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("API error {error_type}: {message}")]
    Api {
        error_type: String,
        message: String,
    },
    #[error("HTTP error: {0}")]
    Http(String),
}

/// Error details from Anthropic API
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub r#type: String,
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub r#type: String,
    pub message: String,
}