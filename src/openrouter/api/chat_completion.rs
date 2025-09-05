use serde::{Deserialize, Serialize};

use crate::openrouter::api::types::Parameters;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProviderPreferences {
    pub sort: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ReasoningConfig {
    pub effort: Option<Effort>,
    pub max_tokens: Option<u32>,
    pub exclude: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UsageConfig {
    pub include: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    #[serde(flatten)]
    pub parameters: Parameters,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub choices: Option<Vec<ChatCompletionChoice>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChoice {
    pub message: Option<ChatCompletionMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: Option<String>,
    pub content: Option<String>,
}
