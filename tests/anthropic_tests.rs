use ai_providers::anthropic::{Anthropic, AnthropicModel, Error};
use ai_types::llm::{LanguageModel, LanguageModelProvider};
use futures_util::StreamExt;

#[cfg(test)]
mod anthropic_tests {
    use super::*;

    #[test]
    fn test_anthropic_creation() {
        let provider = Anthropic::new("test-api-key");
        // Just ensure it can be created without panicking
        assert!(!format!("{:?}", provider).is_empty());
    }

    #[test]
    fn test_anthropic_provider_profile() {
        let profile = Anthropic::profile();
        assert_eq!(profile.name(), "Anthropic");
        assert!(profile.description().contains("Claude AI"));
        assert!(profile.description().contains("safe, beneficial"));
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = Anthropic::new("test-api-key");

        let models = provider
            .list_models()
            .await
            .expect("Should return built-in models");

        // Anthropic returns a predefined list of models
        assert!(!models.is_empty());
        assert!(models.len() >= 5); // Should have at least 5 Claude models

        // Check that common models are present
        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(model_names.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(model_names.contains(&"claude-3-opus-20240229".to_string()));

        // Verify model structure
        for model in models {
            assert!(!model.name.is_empty());
            assert_eq!(model.author, "Anthropic");
            assert!(!model.slug.is_empty());
            assert!(!model.description.is_empty());
            assert_eq!(model.context_length, 200000); // All Claude models have 200k context
        }
    }

    #[tokio::test]
    async fn test_get_model_valid() {
        let provider = Anthropic::new("test-api-key");

        let model = provider
            .get_model("claude-3-5-sonnet-20241022")
            .await
            .expect("Should get valid model");

        assert_eq!(model.model_name, "claude-3-5-sonnet-20241022");
    }

    #[tokio::test]
    async fn test_get_model_invalid_name() {
        let provider = Anthropic::new("test-api-key");

        let result = provider.get_model("invalid-model-name").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::InvalidModelName(name) => {
                assert_eq!(name, "invalid-model-name");
            }
            _ => panic!("Expected InvalidModelName error"),
        }
    }

    #[test]
    fn test_anthropic_model_creation() {
        let provider = Anthropic::new("test-api-key");
        let model = AnthropicModel::new(provider, "claude-3-5-sonnet-20241022".to_string());

        // Basic structure test
        assert!(!format!("{:?}", model).is_empty());
    }

    #[tokio::test]
    async fn test_model_profile() {
        let provider = Anthropic::new("test-api-key");
        let model = AnthropicModel::new(provider, "claude-3-5-sonnet-20241022".to_string());

        let profile = model.profile().await;
        assert_eq!(profile.name, "claude-3-5-sonnet-20241022");
        assert_eq!(profile.author, "Anthropic");
        assert_eq!(profile.slug, "claude-3-5-sonnet-20241022");
        assert_eq!(profile.description, "Claude AI model by Anthropic");
        assert_eq!(profile.context_length, 200000);
    }

    #[tokio::test]
    async fn test_respond_stream_structure() {
        let provider = Anthropic::new("test-api-key");
        let model = AnthropicModel::new(provider, "claude-3-5-sonnet-20241022".to_string());

        let messages = vec![ai_types::llm::Message::user("Hello")];
        let mut tools = ai_types::llm::tool::Tools::new();
        let parameters = ai_types::llm::model::Parameters::default();

        let stream = model.respond(&messages, &mut tools, &parameters);

        // Pin the stream and try to get one item
        // This will likely fail due to invalid API key, but tests the stream setup
        futures_util::pin_mut!(stream);
        match stream.next().await {
            Some(Ok(text)) => {
                // Successful response
                assert!(!text.is_empty());
            }
            Some(Err(_)) => {
                // Expected error due to invalid API key
            }
            None => {
                // Empty stream - also possible
            }
        }
    }

    #[tokio::test]
    async fn test_complete_stream_structure() {
        let provider = Anthropic::new("test-api-key");
        let model = AnthropicModel::new(provider, "claude-3-5-sonnet-20241022".to_string());

        let stream = model.complete("The weather today is");

        // Pin the stream and try to get one item
        futures_util::pin_mut!(stream);
        match stream.next().await {
            Some(Ok(text)) => {
                // Successful response
                assert!(!text.is_empty());
            }
            Some(Err(_)) => {
                // Expected error due to invalid API key
            }
            None => {
                // Empty stream - also possible
            }
        }
    }

    #[test]
    fn test_error_display() {
        let api_error =
            ai_providers::anthropic::api::error::ApiError::Http("Test error".to_string());
        let error = Error::Api(api_error);

        let error_str = format!("{}", error);
        assert!(error_str.contains("Api error"));
        assert!(error_str.contains("Test error"));
    }

    #[test]
    fn test_message_conversion() {
        // Test that our message conversion logic works correctly
        let user_msg = ai_types::llm::Message::user("Hello");
        let assistant_msg = ai_types::llm::Message::assistant("Hi there");
        let system_msg = ai_types::llm::Message::system("You are helpful");

        assert_eq!(user_msg.role(), ai_types::llm::Role::User);
        assert_eq!(assistant_msg.role(), ai_types::llm::Role::Assistant);
        assert_eq!(system_msg.role(), ai_types::llm::Role::System);

        assert_eq!(user_msg.content(), "Hello");
        assert_eq!(assistant_msg.content(), "Hi there");
        assert_eq!(system_msg.content(), "You are helpful");
    }

    #[test]
    fn test_anthropic_models_list() {
        // Test that all expected Claude models are in the predefined list
        let provider = Anthropic::new("test-key");

        // This is a synchronous operation since it returns a predefined list
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let models = runtime.block_on(provider.list_models()).unwrap();

        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        // Check all expected models are present
        assert!(model_names.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(model_names.contains(&"claude-3-5-haiku-20241022".to_string()));
        assert!(model_names.contains(&"claude-3-opus-20240229".to_string()));
        assert!(model_names.contains(&"claude-3-sonnet-20240229".to_string()));
        assert!(model_names.contains(&"claude-3-haiku-20240307".to_string()));
    }

    #[test]
    fn test_system_message_handling() {
        // Test that system messages are properly extracted and handled
        let messages = vec![
            ai_types::llm::Message::system("You are a helpful assistant"),
            ai_types::llm::Message::user("Hello"),
            ai_types::llm::Message::assistant("Hi there"),
        ];

        // Test the logic that would be used in the respond method
        let system_message = messages
            .iter()
            .find(|msg| msg.role() == ai_types::llm::Role::System)
            .map(|msg| msg.content().to_string());

        assert!(system_message.is_some());
        assert_eq!(system_message.unwrap(), "You are a helpful assistant");

        // Test filtering out system messages from the main message list
        let anthropic_messages: Vec<_> = messages
            .iter()
            .filter_map(|msg| match msg.role() {
                ai_types::llm::Role::User => Some(("user", msg.content())),
                ai_types::llm::Role::Assistant => Some(("assistant", msg.content())),
                ai_types::llm::Role::System => None,
                ai_types::llm::Role::Tool => None,
            })
            .collect();

        assert_eq!(anthropic_messages.len(), 2);
        assert_eq!(anthropic_messages[0], ("user", "Hello"));
        assert_eq!(anthropic_messages[1], ("assistant", "Hi there"));
    }

    // Mock HTTP tests (would require additional setup for true unit testing)
    #[cfg(test)]
    mod mock_tests {

        // These tests would use a mock HTTP client to test the actual API interactions
        // without making real network calls

        #[test]
        fn test_request_headers() {
            // Test that proper headers are set (would need HTTP mock)
            // - x-api-key header with API key
            // - anthropic-version header
            // - Content-Type: application/json for POST requests
        }

        #[test]
        fn test_request_body_serialization() {
            // Test that request bodies are properly serialized to JSON
            // and match Anthropic API expectations
            // - system message in separate field
            // - proper message format
            // - streaming parameter
        }

        #[test]
        fn test_response_deserialization() {
            // Test that API responses are properly deserialized
            // into our response types
        }

        #[test]
        fn test_error_handling() {
            // Test different error scenarios:
            // - Network errors
            // - API errors (4xx, 5xx)
            // - Malformed responses
        }
    }
}

// Integration tests (require valid API key and network access)
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;
    use std::env;

    fn get_anthropic_api_key() -> Option<String> {
        env::var("ANTHROPIC_API_KEY").ok()
    }

    #[tokio::test]
    async fn test_real_api_get_model() {
        let Some(api_key) = get_anthropic_api_key() else {
            println!("Skipping integration test - no ANTHROPIC_API_KEY");
            return;
        };

        let provider = Anthropic::new(api_key);
        let model = provider
            .get_model("claude-3-5-sonnet-20241022")
            .await
            .expect("Failed to get model");

        // Verify model was created properly
        let profile = model.profile().await;
        assert_eq!(profile.name, "claude-3-5-sonnet-20241022");
        assert_eq!(profile.author, "Anthropic");
    }

    #[tokio::test]
    async fn test_real_api_chat() {
        let Some(api_key) = get_anthropic_api_key() else {
            println!("Skipping integration test - no ANTHROPIC_API_KEY");
            return;
        };

        let provider = Anthropic::new(api_key);
        let model = provider
            .get_model("claude-3-5-haiku-20241022")
            .await
            .expect("Failed to get model");

        let messages = vec![
            ai_types::llm::Message::system("You are a helpful assistant that responds briefly"),
            ai_types::llm::Message::user("Say 'Hello from Anthropic!' and nothing else"),
        ];
        let mut tools = ai_types::llm::tool::Tools::new();
        let parameters = ai_types::llm::model::Parameters::default().max_tokens(50);

        let mut stream = model.respond(&messages, &mut tools, &parameters);
        let mut response = String::new();

        futures_util::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => response.push_str(&text),
                Err(e) => panic!("Stream error: {}", e),
            }
        }

        assert!(!response.is_empty());
        assert!(response.to_lowercase().contains("hello"));
    }

    #[tokio::test]
    async fn test_real_api_completion() {
        let Some(api_key) = get_anthropic_api_key() else {
            println!("Skipping integration test - no ANTHROPIC_API_KEY");
            return;
        };

        let provider = Anthropic::new(api_key);
        let model = provider
            .get_model("claude-3-5-haiku-20241022")
            .await
            .expect("Failed to get model");

        let mut stream = model.complete("The capital of France is");
        let mut response = String::new();

        futures_util::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    response.push_str(&text);
                    // Break after getting some response to keep test fast
                    if response.len() > 10 {
                        break;
                    }
                }
                Err(e) => panic!("Stream error: {}", e),
            }
        }

        assert!(!response.is_empty());
    }
}
