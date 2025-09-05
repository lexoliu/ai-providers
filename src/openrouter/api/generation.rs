use serde::{Deserialize, Serialize};
use zenwave::{Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub id: String,
    pub total_cost: f64,
    pub created_at: String,
    pub model: String,
    pub origin: String,
    pub usage: f64,
    pub is_byok: bool,
    pub upstream_id: Option<String>,
    pub cache_discount: Option<f64>,
    pub upstream_inference_cost: Option<f64>,
    pub app_id: Option<i64>,
    pub streamed: Option<bool>,
    pub cancelled: Option<bool>,
    pub provider_name: Option<String>,
    pub latency: Option<i64>,
    pub moderation_latency: Option<i64>,
    pub generation_time: Option<i64>,
    pub finish_reason: Option<String>,
    pub native_finish_reason: Option<String>,
    pub tokens_prompt: Option<i64>,
    pub tokens_completion: Option<i64>,
    pub native_tokens_prompt: Option<i64>,
    pub native_tokens_completion: Option<i64>,
    pub native_tokens_reasoning: Option<i64>,
    pub num_media_prompt: Option<i64>,
    pub num_media_completion: Option<i64>,
    pub num_search_results: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationMetadataResponse {
    pub data: GenerationMetadata,
}

pub async fn get_generation_metadata(
    client: &mut impl Client,
    base_url: &str,
    token: &str,
    id: &str,
) -> Result<GenerationMetadataResponse, Error> {
    let url = format!("{}/generation/metadata", base_url);
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .query(&[("id", id)])
        .send()
        .await?
        .error_for_status()?
        .json::<GenerationMetadataResponse>()
        .await?;
    Ok(resp)
}
