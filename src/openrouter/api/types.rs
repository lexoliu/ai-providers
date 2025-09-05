use serde::{Deserialize, Serialize};

fn str_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn str_to_option_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

fn default_f64() -> f64 {
    0.0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pricing {
    #[serde(deserialize_with = "str_to_f64")]
    pub prompt: f64,
    #[serde(deserialize_with = "str_to_f64")]
    pub completion: f64,
    #[serde(deserialize_with = "str_to_f64", default = "default_f64")]
    pub image: f64,
    #[serde(deserialize_with = "str_to_f64", default = "default_f64")]
    pub request: f64,
    #[serde(deserialize_with = "str_to_f64", default = "default_f64")]
    pub web_search: f64,
    #[serde(deserialize_with = "str_to_f64", default = "default_f64")]
    pub internal_reasoning: f64,
    #[serde(deserialize_with = "str_to_option_f64", default)]
    pub input_cache_read: Option<f64>,
    #[serde(deserialize_with = "str_to_option_f64", default)]
    pub input_cache_write: Option<f64>,
}

impl From<Pricing> for ai_types::llm::model::Pricing {
    fn from(pricing: Pricing) -> Self {
        let mut prciing = ai_types::llm::model::Pricing::default();
        prciing.prompt = pricing.prompt;
        prciing.completion = pricing.completion;
        prciing.image = pricing.image;
        prciing.request = pricing.request;
        prciing.web_search = pricing.web_search;
        prciing.internal_reasoning = pricing.internal_reasoning;
        prciing.input_cache_read = pricing.input_cache_read.unwrap_or_default();
        prciing.input_cache_write = pricing.input_cache_write.unwrap_or_default();
        prciing
    }
}

use serde_json::Value;
use std::collections::BTreeMap;

/// Parameters for configuring OpenRouter API requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Parameters {
    /// Influences the variety in the model's responses. 0.0 to 2.0, default 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Limits the model's choices to a percentage of likely tokens. 0.0 to 1.0, default 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Limits the model's choice of tokens at each step. 0 or above, default 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Controls repetition of tokens based on frequency. -2.0 to 2.0, default 0.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Adjusts repetition of tokens already used in the input. -2.0 to 2.0, default 0.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Reduces repetition of tokens from the input. 0.0 to 2.0, default 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,

    /// Minimum probability for a token to be considered. 0.0 to 1.0, default 0.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,

    /// Only consider top tokens with sufficiently high probabilities. 0.0 to 1.0, default 0.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f32>,

    /// If specified, sampling will be deterministic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,

    /// Upper limit for the number of tokens the model can generate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Maps token IDs to bias values (-100 to 100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<BTreeMap<String, f32>>,

    /// Whether to return log probabilities of the output tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of most likely tokens to return at each token position (0-20).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// Forces the model to produce specific output format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<BTreeMap<String, Value>>,

    /// If the model can return structured outputs using response_format json_schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_outputs: Option<bool>,

    /// Stop generation if the model encounters any token in this array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Tool calling parameter, following OpenAI's tool calling request shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,

    /// Controls which tool is called by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl From<ai_types::llm::model::Parameters> for Parameters {
    fn from(val: ai_types::llm::model::Parameters) -> Self {
        Parameters {
            temperature: val.temperature,
            top_p: val.top_p,
            top_k: val.top_k,
            frequency_penalty: val.frequency_penalty,
            presence_penalty: val.presence_penalty,
            repetition_penalty: val.repetition_penalty,
            min_p: val.min_p,
            top_a: val.top_a,
            seed: val.seed,
            max_tokens: val.max_tokens,
            logit_bias: val.logit_bias.map(|b| b.into_iter().collect()),
            logprobs: val.logprobs,
            top_logprobs: val.top_logprobs,
            response_format: val.response_format.map(|value| {
                let object = value.as_object().unwrap().clone();
                object.into_iter().collect::<BTreeMap<_, _>>()
            }),
            structured_outputs: None, // set later
            stop: val.stop.map(|s| s.into_iter().collect()),
            tools: None, // Set later
            tool_choice: val.tool_choice,
            stream: Some(false), // We don't want streaming for now
            user: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Architecture {
    #[serde(default)]
    input_modalities: Option<Vec<String>>,
    #[serde(default)]
    output_modalities: Option<Vec<String>>,
    tokenizer: String,
    #[serde(default)]
    instruct_type: Option<String>,
}
