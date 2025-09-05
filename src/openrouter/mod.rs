use ai_types::{
    LanguageModel,
    llm::{LanguageModelProvider, model, provider},
};
pub mod api;
use crate::openrouter::api::list_models::ListModelsResponse;
use futures_util::Stream;
use serde::{Serialize, de::DeserializeOwned};
use zenwave::Method;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Api error: {0}")]
    Api(#[from] api::error::ApiError),
    #[error("Invalid model name: {0}")]
    InvalidModelName(String),
}

#[derive(Clone, Debug)]
pub struct OpenRouter {
    token: String,
}

impl OpenRouter {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    pub async fn request<Req: Serialize, Res: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        req: Req,
    ) -> Result<Res, Error> {
        use zenwave::Client;

        let base_url = "https://openrouter.ai/api/v1";
        let url = format!("{}{}", base_url, path);

        let mut client = zenwave::client();

        let mut request = client.method(method.clone(), url);

        // Add OpenRouter-specific headers
        request = request
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json");

        // For methods that send data, add JSON body
        if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
            request = request
                .json_body(&req)
                .map_err(|e| api::error::ApiError::Http(e.to_string()))?;
        }

        // Try to get the JSON response, which will handle errors properly
        match request.json::<Res>().await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Check if this is an HTTP status error by trying to parse the error message
                let error_msg = e.to_string().to_lowercase();

                // Handle specific OpenRouter error formats and status codes
                if error_msg.contains("401")
                    || error_msg.contains("unauthorized")
                    || error_msg.contains("invalid key")
                    || error_msg.contains("invalid api key")
                    || error_msg.contains("missing field")
                    || error_msg.contains("authentication")
                {
                    Err(api::error::ApiError::Api {
                        code: 401,
                        message:
                            "Invalid API key. Please check your OpenRouter API key and try again."
                                .to_string(),
                        metadata: None,
                    }
                    .into())
                } else if error_msg.contains("expected string, received null")
                    || error_msg.contains("\"code\":400")
                {
                    Err(api::error::ApiError::Api {
                        code: 400,
                        message: "OpenRouter API request error. This may be due to an invalid API key or malformed request parameters.".to_string(),
                        metadata: None,
                    }.into())
                } else if error_msg.contains("403") || error_msg.contains("forbidden") {
                    Err(api::error::ApiError::Api {
                        code: 403,
                        message:
                            "Access forbidden. Please check your OpenRouter API key permissions."
                                .to_string(),
                        metadata: None,
                    }
                    .into())
                } else if error_msg.contains("429") || error_msg.contains("rate") {
                    Err(api::error::ApiError::Api {
                        code: 429,
                        message: "Rate limit exceeded. Please try again later.".to_string(),
                        metadata: None,
                    }
                    .into())
                } else {
                    // Return the original HTTP/JSON parsing error
                    Err(api::error::ApiError::Http(format!("Request failed: {}", e)).into())
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct OpenRouterModel {
    provider: OpenRouter,
    author: String,
    slug: String,
}

impl OpenRouterModel {
    pub const fn new(provider: OpenRouter, author: String, slug: String) -> Self {
        Self {
            provider,
            author,
            slug,
        }
    }
}

impl LanguageModelProvider for OpenRouter {
    type Model = OpenRouterModel;
    type Error = Error;

    async fn list_models(&self) -> Result<Vec<model::Profile>, Error> {
        let response: ListModelsResponse = self.request(Method::GET, "/models", ()).await?;
        Ok(response
            .data
            .into_iter()
            .map(|model| {
                let (author, slug) = model.id.split_once('/').unwrap_or(("", ""));
                model::Profile::new(
                    model.id.clone(),
                    author,
                    slug,
                    model.description,
                    model.top_provider.context_length.unwrap_or(4096),
                )
            })
            .collect::<Vec<_>>())
    }

    async fn get_model(&self, name: &str) -> Result<Self::Model, Error> {
        // Validate the model name format (should be "author/model")
        let (author, slug) = name
            .split_once('/')
            .ok_or_else(|| Error::InvalidModelName(name.to_string()))?;

        // Basic validation - author and slug shouldn't be empty
        if author.is_empty() || slug.is_empty() {
            return Err(Error::InvalidModelName(name.to_string()));
        }

        // Create the model without making an API call
        // The actual validation happens when making requests
        Ok(OpenRouterModel::new(
            self.clone(),
            author.to_string(),
            slug.to_string(),
        ))
    }

    fn profile() -> provider::Profile {
        provider::Profile::new(
            "OpenRouter",
            "Provides a unified API that gives you access to hundreds of AI models through a single endpoint,",
        )
    }
}

impl LanguageModel for OpenRouterModel {
    type Error = Error;

    fn respond(
        &self,
        messages: &[ai_types::llm::Message],
        _tools: &mut ai_types::llm::tool::Tools,
        parameters: &ai_types::llm::model::Parameters,
    ) -> impl Stream<Item = Result<String, Self::Error>> + Send {
        use crate::openrouter::api::chat_completion::{ChatCompletionRequest, Message, Role};

        // Convert ai_types messages to OpenRouter messages, ensuring content is not empty
        let openrouter_messages: Vec<Message> = messages
            .iter()
            .filter_map(|msg| {
                let content = msg.content().trim();
                if content.is_empty() {
                    None // Skip empty messages
                } else {
                    Some(Message {
                        role: match msg.role() {
                            ai_types::llm::Role::User => Role::User,
                            ai_types::llm::Role::Assistant => Role::Assistant,
                            ai_types::llm::Role::System => Role::System,
                            ai_types::llm::Role::Tool => Role::Tool,
                        },
                        content: content.to_string(),
                    })
                }
            })
            .collect();

        let request = ChatCompletionRequest {
            model: format!("{}/{}", self.author, self.slug),
            messages: openrouter_messages,
            parameters: crate::openrouter::api::types::Parameters {
                temperature: parameters.temperature,
                max_tokens: parameters.max_tokens,
                top_p: parameters.top_p,
                frequency_penalty: parameters.frequency_penalty,
                presence_penalty: parameters.presence_penalty,
                stop: parameters.stop.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        let provider = self.provider.clone();

        async_stream::stream! {
            // Ensure we have at least one valid message
            if request.messages.is_empty() {
                yield Err(Error::Api(api::error::ApiError::Api {
                    code: 400,
                    message: "No valid messages provided. All messages were empty or invalid.".to_string(),
                    metadata: None,
                }));
                return;
            }
            match provider.request(Method::POST, "/chat/completions", request).await {
                Ok(response) => {
                    use crate::openrouter::api::chat_completion::ChatCompletionResponse;
                    let resp: ChatCompletionResponse = response;

                    // Check if this might be an authentication error disguised as a successful response
                    if resp.id.is_none() && resp.choices.is_none() {
                        yield Err(Error::Api(api::error::ApiError::Api {
                            code: 401,
                            message: "OpenRouter returned an empty response. This usually indicates an invalid API key or authentication issue.".to_string(),
                            metadata: None,
                        }));
                        return;
                    }

                    if let Some(choices) = resp.choices {
                        if choices.is_empty() {
                            yield Err(Error::Api(api::error::ApiError::Api {
                                code: 500,
                                message: "OpenRouter returned no choices in response. This may indicate a rate limit, content policy violation, or service issue.".to_string(),
                                metadata: None,
                            }));
                            return;
                        }

                        let mut had_content = false;
                        for choice in choices {
                            if let Some(message) = choice.message
                                && let Some(content) = message.content {
                                    if content.is_empty() {
                                        yield Err(Error::Api(api::error::ApiError::Api {
                                            code: 500,
                                            message: "OpenRouter returned empty content. This may indicate a content policy violation or service issue.".to_string(),
                                            metadata: None,
                                        }));
                                    } else {
                                        had_content = true;
                                        yield Ok(content);
                                    }
                                }
                        }

                        if !had_content {
                            yield Err(Error::Api(api::error::ApiError::Api {
                                code: 500,
                                message: "OpenRouter returned choices without message content.".to_string(),
                                metadata: None,
                            }));
                        }
                    } else {
                        yield Err(Error::Api(api::error::ApiError::Api {
                            code: 401,
                            message: "OpenRouter returned null choices. This usually indicates an invalid API key or authentication issue. Please check your API key.".to_string(),
                            metadata: None,
                        }));
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
        let author = self.author.clone();
        let slug = self.slug.clone();

        async_stream::stream! {
            // OpenRouter doesn't typically support completion endpoints, so we'll convert this to a chat completion
            let messages = vec![ai_types::llm::Message::user(prefix)];
            let mut tools = ai_types::llm::tool::Tools::new();
            let parameters = ai_types::llm::model::Parameters::default();

            let model = OpenRouterModel { provider, author, slug };
            let stream = model.respond(&messages, &mut tools, &parameters);

            // Pin the stream to make it Unpin
            futures_util::pin_mut!(stream);

            while let Some(result) = stream.next().await {
                yield result;
            }
        }
    }

    async fn profile(&self) -> model::Profile {
        model::Profile::new(
            format!("{}/{}", self.author, self.slug),
            self.author.clone(),
            self.slug.clone(),
            "OpenRouter model".to_string(),
            32768, // Default context length, could be fetched from API
        )
    }
}
