use ai_types::{
    LanguageModel,
    llm::{LanguageModelProvider, model, provider},
};
pub mod api;
// API types are imported in individual functions where needed
use futures_util::Stream;
use serde::{Serialize, de::DeserializeOwned};
use zenwave::Method;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Api error: {0}")]
    Api(#[from] api::error::ApiError),
    #[error("Invalid model name: {0}")]
    InvalidModelName(String),
    #[error("HTTP error: {0}")]
    Http(#[from] zenwave::Error),
}

#[derive(Clone, Debug)]
pub struct Anthropic {
    api_key: String,
}

impl Anthropic {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    pub async fn request<Req: Serialize, Res: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        req: Req,
    ) -> Result<Res, Error> {
        use zenwave::Client;

        let base_url = "https://api.anthropic.com/v1";
        let url = format!("{}{}", base_url, path);

        let mut client = zenwave::client();

        let mut request = client.method(method.clone(), url);

        // Add Anthropic-specific headers
        request = request
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01");

        // For methods that send data, add JSON body
        if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
            request = request.json_body(&req)?;
        }

        let response = request.json::<Res>().await?;

        Ok(response)
    }
}

#[derive(Debug)]
pub struct AnthropicModel {
    provider: Anthropic,
    pub model_name: String,
}

impl AnthropicModel {
    pub const fn new(provider: Anthropic, model_name: String) -> Self {
        Self {
            provider,
            model_name,
        }
    }
}

impl LanguageModelProvider for Anthropic {
    type Model = AnthropicModel;
    type Error = Error;

    async fn list_models(&self) -> Result<Vec<model::Profile>, Error> {
        // Anthropic doesn't have a public models API, so we'll return common models
        Ok(vec![
            model::Profile::new(
                "claude-3-5-sonnet-20241022",
                "Anthropic",
                "claude-3-5-sonnet-20241022",
                "Most intelligent model, combining top-level intelligence with improved speed",
                200000,
            ),
            model::Profile::new(
                "claude-3-5-haiku-20241022",
                "Anthropic",
                "claude-3-5-haiku-20241022",
                "Fastest model for daily tasks, with improved instruction following",
                200000,
            ),
            model::Profile::new(
                "claude-3-opus-20240229",
                "Anthropic",
                "claude-3-opus-20240229",
                "Most powerful model for complex tasks",
                200000,
            ),
            model::Profile::new(
                "claude-3-sonnet-20240229",
                "Anthropic",
                "claude-3-sonnet-20240229",
                "Balanced model for scaled deployments",
                200000,
            ),
            model::Profile::new(
                "claude-3-haiku-20240307",
                "Anthropic",
                "claude-3-haiku-20240307",
                "Fastest model for lightweight tasks",
                200000,
            ),
        ])
    }

    async fn get_model(&self, name: &str) -> Result<Self::Model, Error> {
        // For Anthropic, we just validate that it's a known model name
        let valid_models = [
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
        ];

        if !valid_models.contains(&name) {
            return Err(Error::InvalidModelName(name.to_string()));
        }

        Ok(AnthropicModel::new(self.clone(), name.to_string()))
    }

    fn profile() -> provider::Profile {
        provider::Profile::new(
            "Anthropic",
            "Claude AI models by Anthropic for safe, beneficial, and understandable AI systems",
        )
    }
}

impl LanguageModel for AnthropicModel {
    type Error = Error;

    fn respond(
        &self,
        messages: &[ai_types::llm::Message],
        _tools: &mut ai_types::llm::tool::Tools,
        parameters: &ai_types::llm::model::Parameters,
    ) -> impl Stream<Item = Result<String, Self::Error>> + Send {
        use crate::anthropic::api::messages::{Message, MessagesRequest, Role};

        // Convert ai_types messages to Anthropic messages
        let anthropic_messages: Vec<Message> = messages
            .iter()
            .filter_map(|msg| {
                match msg.role() {
                    ai_types::llm::Role::User => Some(Message {
                        role: Role::User,
                        content: msg.content().to_string(),
                    }),
                    ai_types::llm::Role::Assistant => Some(Message {
                        role: Role::Assistant,
                        content: msg.content().to_string(),
                    }),
                    // Anthropic handles system messages differently - they go in the system field
                    ai_types::llm::Role::System => None,
                    ai_types::llm::Role::Tool => None, // TODO: Handle tool messages
                }
            })
            .collect();

        // Extract system message if any
        let system_message = messages
            .iter()
            .find(|msg| msg.role() == ai_types::llm::Role::System)
            .map(|msg| msg.content().to_string());

        let request = MessagesRequest {
            model: self.model_name.clone(),
            messages: anthropic_messages,
            max_tokens: parameters.max_tokens.unwrap_or(1024),
            system: system_message,
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            top_k: parameters.top_k,
            stop_sequences: parameters.stop.clone(),
            stream: Some(true),
        };

        let provider = self.provider.clone();

        async_stream::stream! {
            match provider.request(Method::POST, "/messages", request).await {
                Ok(response) => {
                    use crate::anthropic::api::messages::MessagesResponse;
                    let resp: MessagesResponse = response;

                    if let Some(content) = resp.content {
                        for content_block in content {
                            if let Some(text) = content_block.text {
                                yield Ok(text);
                            }
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

            let model = AnthropicModel { provider, model_name };
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
            "Anthropic",
            self.model_name.clone(),
            "Claude AI model by Anthropic",
            200000,
        )
    }
}
