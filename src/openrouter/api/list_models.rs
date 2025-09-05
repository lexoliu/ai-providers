use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::openrouter::api::types::{Architecture, Pricing};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListModelsQuery {
    pub category: Option<String>,
    pub use_rss: Option<bool>,
    pub use_rss_chat_links: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopProvider {
    #[serde(default)]
    pub is_moderated: Option<bool>,
    #[serde(default)]
    pub context_length: Option<u32>,
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    // Allow for unknown fields
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub architecture: Architecture,
    pub top_provider: TopProvider,
    pub pricing: Pricing,
    #[serde(default)]
    pub canonical_slug: Option<String>,
    #[serde(default)]
    pub hugging_face_id: Option<String>,
    #[serde(default)]
    pub per_request_limits: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub supported_parameters: Option<Vec<String>>,
    // Allow for unknown fields that OpenRouter might add in the future
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListModelsResponse {
    pub data: Vec<Model>,
}
