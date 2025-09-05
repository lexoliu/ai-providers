use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main error object returned by the API.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("API error {code}: {message}")]
    Api {
        code: u16,
        message: String,
        #[serde(default)]
        metadata: Option<ErrorMetadata>,
    },
    #[error("HTTP error: {0}")]
    Http(String),
}


/// Metadata for different error types.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorMetadata {
    Moderation(ModerationErrorMetadata),
    Provider(ProviderErrorMetadata),
    Other(HashMap<String, serde_json::Value>),
}

/// Metadata for moderation errors.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationErrorMetadata {
    pub reasons: Vec<String>,
    pub flagged_input: String,
    pub provider_name: String,
    pub model_slug: String,
}

/// Metadata for provider errors.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderErrorMetadata {
    pub provider_name: String,
    pub raw: serde_json::Value,
}
