use serde::{Deserialize, Serialize};

use crate::openrouter::api::types::Architecture;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListEndpointResponse {
    pub data: ModelData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelData {
    pub id: String,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub architecture: Architecture,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub name: String,
    pub context_length: f64,
    pub pricing: Pricing,
    pub request: String,
    pub image: String,
    pub prompt: String,
    pub completion: String,
    pub provider_name: String,
    pub supported_parameters: Vec<String>,
    pub quantization: Option<String>,
    pub max_completion_tokens: Option<f64>,
    pub max_prompt_tokens: Option<f64>,
    pub status: Option<String>,
    pub uptime_last_30m: Option<f64>, // Rolling 30-minute uptime as a percentage, null if <100 requests
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pricing {
    // Add pricing fields as needed
}
