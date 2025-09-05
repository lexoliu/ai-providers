use crate::shared::openai_api::{
    ChatCompletionRequest, ChatCompletionResponse, ModelsResponse, apply_parameters,
    convert_message,
};
use ai_types::{
    LanguageModel,
    llm::{LanguageModelProvider, model, provider},
};
use futures_util::Stream;
use serde::{Serialize, de::DeserializeOwned};
use zenwave::Method;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OpenAI API error: {0}")]
    Api(#[from] crate::shared::openai_api::ApiError),
    #[error("HTTP error: {0}")]
    Http(#[from] zenwave::Error),
    #[error("Invalid model name: {0}")]
    InvalidModelName(String),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct OpenAI {
    api_key: String,
    base_url: String,
}

impl OpenAI {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub async fn request<Req: Serialize, Res: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        req: Req,
    ) -> Result<Res, Error> {
        use zenwave::Client;

        let url = format!("{}{}", self.base_url, path);

        let mut client = zenwave::client();

        let mut request = client.method(method.clone(), url);

        // Add OpenAI-specific headers
        request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // For methods that send data, add JSON body
        if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
            request = request.json_body(&req)?;
        }

        // Try to get the JSON response, which will handle errors properly
        match request.json::<Res>().await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Check if this is an HTTP status error by trying to parse the error message
                let error_msg = e.to_string();
                
                // If it contains status codes or API key related errors
                if error_msg.contains("401") || error_msg.contains("Unauthorized") || 
                   error_msg.contains("missing field `id`") || error_msg.contains("invalid api key") {
                    Err(Error::Api(crate::shared::openai_api::ApiError {
                        message: "Invalid API key. Please check your OpenAI API key and try again.".to_string(),
                        r#type: "invalid_request_error".to_string(),
                        param: Some("api_key".to_string()),
                        code: Some("invalid_api_key".to_string()),
                    }))
                } else if error_msg.contains("403") || error_msg.contains("Forbidden") {
                    Err(Error::Api(crate::shared::openai_api::ApiError {
                        message: "Access forbidden. Please check your API key permissions.".to_string(),
                        r#type: "invalid_request_error".to_string(),
                        param: Some("api_key".to_string()),
                        code: Some("forbidden".to_string()),
                    }))
                } else if error_msg.contains("429") || error_msg.contains("rate") {
                    Err(Error::Api(crate::shared::openai_api::ApiError {
                        message: "Rate limit exceeded. Please try again later.".to_string(),
                        r#type: "rate_limit_error".to_string(),
                        param: None,
                        code: Some("rate_limit_exceeded".to_string()),
                    }))
                } else {
                    // Return the original HTTP/JSON parsing error
                    Err(Error::Http(e))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct OpenAIModel {
    provider: OpenAI,
    pub model_name: String,
}

impl OpenAIModel {
    pub const fn new(provider: OpenAI, model_name: String) -> Self {
        Self {
            provider,
            model_name,
        }
    }
}

impl LanguageModelProvider for OpenAI {
    type Model = OpenAIModel;
    type Error = Error;

    async fn list_models(&self) -> Result<Vec<model::Profile>, Error> {
        let response: ModelsResponse = self.request(Method::GET, "/models", ()).await?;

        Ok(response
            .data
            .into_iter()
            .map(|model| {
                // Parse owner and model name
                let parts: Vec<&str> = model.id.split('/').collect();
                let (owner, name) = if parts.len() > 1 {
                    (parts[0], parts[1])
                } else {
                    ("openai", model.id.as_str())
                };

                model::Profile::new(
                    model.id.clone(),
                    owner,
                    name,
                    format!("OpenAI model: {}", model.id),
                    get_context_length(&model.id),
                )
            })
            .collect())
    }

    async fn get_model(&self, name: &str) -> Result<Self::Model, Error> {
        // For OpenAI, we just validate that the model name looks reasonable
        // The actual validation happens when making API calls
        if name.is_empty() {
            return Err(Error::InvalidModelName(name.to_string()));
        }

        Ok(OpenAIModel::new(self.clone(), name.to_string()))
    }

    fn profile() -> provider::Profile {
        provider::Profile::new(
            "OpenAI",
            "Official OpenAI API for GPT models and other AI services",
        )
    }
}

impl LanguageModel for OpenAIModel {
    type Error = Error;

    fn respond(
        &self,
        messages: &[ai_types::llm::Message],
        _tools: &mut ai_types::llm::tool::Tools,
        parameters: &ai_types::llm::model::Parameters,
    ) -> impl Stream<Item = Result<String, Self::Error>> + Send {
        // Convert ai_types messages to OpenAI API messages
        let openai_messages: Vec<_> = messages.iter().map(convert_message).collect();

        let mut request = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: openai_messages,
            stream: Some(false),
            ..Default::default()
        };

        // Apply parameters
        apply_parameters(&mut request, parameters);

        let provider = self.provider.clone();

        async_stream::stream! {
            match provider.request(Method::POST, "/chat/completions", request).await {
                Ok(response) => {
                    let resp: ChatCompletionResponse = response;

                    if resp.choices.is_empty() {
                        yield Err(Error::Api(crate::shared::openai_api::ApiError {
                            message: "API returned no choices in response. This may indicate a rate limit, content policy violation, or service issue.".to_string(),
                            r#type: "empty_response".to_string(),
                            param: None,
                            code: None,
                        }));
                        return;
                    }

                    if let Some(choice) = resp.choices.first() {
                        if let Some(message) = &choice.message {
                            if message.content.is_empty() {
                                yield Err(Error::Api(crate::shared::openai_api::ApiError {
                                    message: "API returned empty content. This may indicate a content policy violation or service issue.".to_string(),
                                    r#type: "empty_content".to_string(),
                                    param: None,
                                    code: None,
                                }));
                            } else {
                                yield Ok(message.content.clone());
                            }
                        } else {
                            yield Err(Error::Api(crate::shared::openai_api::ApiError {
                                message: "API returned choice without message content.".to_string(),
                                r#type: "missing_message".to_string(),
                                param: None,
                                code: None,
                            }));
                        }
                    }
                }
                Err(e) => yield Err(e),
            }
        }
    }

    fn complete(&self, prefix: &str) -> impl Stream<Item = Result<String, Self::Error>> + Send {
        use futures_util::StreamExt;

        let prefix = prefix.to_owned();
        let provider = self.provider.clone();
        let model_name = self.model_name.clone();

        async_stream::stream! {
            // Convert completion to a chat completion
            let messages = vec![ai_types::llm::Message::user(prefix)];
            let mut tools = ai_types::llm::tool::Tools::new();
            let parameters = ai_types::llm::model::Parameters::default();

            let model = OpenAIModel { provider, model_name };
            let stream = model.respond(&messages, &mut tools, &parameters);

            futures_util::pin_mut!(stream);

            while let Some(result) = stream.next().await {
                yield result;
            }
        }
    }

    async fn profile(&self) -> model::Profile {
        model::Profile::new(
            self.model_name.clone(),
            "OpenAI",
            self.model_name.clone(),
            format!("OpenAI {} model", self.model_name),
            get_context_length(&self.model_name),
        )
    }
}

/// Get approximate context length for known OpenAI models
fn get_context_length(model_name: &str) -> u32 {
    match model_name {
        // GPT-4 models
        "gpt-4" | "gpt-4-0613" => 8192,
        "gpt-4-32k" | "gpt-4-32k-0613" => 32768,
        "gpt-4-1106-preview" | "gpt-4-vision-preview" => 128000,
        "gpt-4-turbo" | "gpt-4-turbo-2024-04-09" => 128000,
        "gpt-4o" | "gpt-4o-2024-05-13" | "gpt-4o-2024-08-06" => 128000,
        "gpt-4o-mini" | "gpt-4o-mini-2024-07-18" => 128000,

        // GPT-3.5 models
        "gpt-3.5-turbo" | "gpt-3.5-turbo-0613" | "gpt-3.5-turbo-1106" => 4096,
        "gpt-3.5-turbo-16k" | "gpt-3.5-turbo-16k-0613" => 16384,
        "gpt-3.5-turbo-instruct" => 4096,

        // Other models
        "text-davinci-003" => 4096,
        "text-davinci-002" => 4096,
        "code-davinci-002" => 8000,

        // Default for unknown models
        _ => 4096,
    }
}
