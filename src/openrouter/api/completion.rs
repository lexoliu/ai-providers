use serde::{Deserialize, Serialize};

use crate::openrouter::api::types::Parameters;

#[derive(Serialize, Debug, Default)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub models: Option<Vec<String>>,
    /// Preferences for provider routing.
    #[serde(default)]
    pub provider: Option<ProviderPreferences>,
    /// Configuration for model reasoning/thinking tokens
    #[serde(default)]
    pub reasoning: Option<ReasoningConfig>,
    /// Whether to include usage information in the response
    #[serde(default)]
    pub usage: Option<UsageConfig>,
    #[serde(default)]
    pub transforms: Option<Vec<String>>,
    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    #[serde(default)]
    pub user: Option<String>,
    #[serde(flatten)]
    pub parameters: Parameters,
}

#[derive(Serialize, Debug)]
pub struct ProviderPreferences {
    sort: SortPreferences,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SortPreferences {
    Price,
    Throughput,
}

#[derive(Serialize, Debug)]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting

    #[serde(default)]
    pub effort: Option<ReasoningEffort>,
    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.

    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Whether to exclude reasoning from the response
    #[serde(default)]
    pub exclude: Option<bool>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
    Medium,
    Low,
}

#[derive(Serialize, Debug)]
pub struct UsageConfig {
    /// Whether to include usage information in the response
    pub include: bool,
}

#[derive(Deserialize, Debug)]
pub struct CompletionResponse {
    pub id: Option<String>,
    pub choices: Option<Vec<CompletionChoice>>,
}

#[derive(Deserialize, Debug)]
pub struct CompletionChoice {
    pub text: Option<String>,
    pub index: Option<u32>,
    pub finish_reason: Option<String>,
}
