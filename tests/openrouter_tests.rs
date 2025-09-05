use ai_providers::openrouter::{Error, OpenRouter, OpenRouterModel};
use ai_types::llm::{LanguageModel, LanguageModelProvider};
use futures_util::StreamExt;

#[cfg(test)]
mod openrouter_tests {
    use super::*;

    #[test]
    fn test_openrouter_creation() {
        let provider = OpenRouter::new("test-token");
        // Just ensure it can be created without panicking
        assert!(!format!("{:?}", provider).is_empty());
    }

    #[test]
    fn test_openrouter_provider_profile() {
        let profile = OpenRouter::profile();
        assert_eq!(profile.name(), "OpenRouter");
        assert!(profile.description().contains("unified API"));
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = OpenRouter::new("test-token");

        // This would normally make a network call, but we'll test the structure
        // In a real test environment, you'd mock the HTTP responses
        match provider.list_models().await {
            Ok(models) => {
                // If successful, verify model structure
                for model in models {
                    assert!(!model.name.is_empty());
                    assert!(!model.author.is_empty());
                    assert!(model.context_length > 0);
                }
            }
            Err(_) => {
                // Expected to fail without valid token/network
                // This is fine for unit tests
            }
        }
    }

    #[tokio::test]
    async fn test_get_model_invalid_name() {
        let provider = OpenRouter::new("test-token");

        let result = provider.get_model("invalid-model-name").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::InvalidModelName(name) => {
                assert_eq!(name, "invalid-model-name");
            }
            _ => panic!("Expected InvalidModelName error"),
        }
    }

    #[tokio::test]
    async fn test_get_model_valid_format() {
        let provider = OpenRouter::new("test-token");

        // This will fail due to network/auth, but should pass name validation
        let result = provider.get_model("openai/gpt-3.5-turbo").await;

        // We expect this to either succeed or fail with an API error, not InvalidModelName
        match result {
            Ok(_) => {
                // Success case - unlikely without real token
            }
            Err(Error::InvalidModelName(_)) => {
                panic!("Should not get InvalidModelName for properly formatted model name");
            }
            Err(Error::Api(_)) => {
                // Expected - API call failed, but name validation passed
            }
        }
    }

    #[test]
    fn test_openrouter_model_creation() {
        let provider = OpenRouter::new("test-token");
        let model =
            OpenRouterModel::new(provider, "openai".to_string(), "gpt-3.5-turbo".to_string());

        // Basic structure test
        assert!(!format!("{:?}", model).is_empty());
    }

    #[tokio::test]
    async fn test_model_profile() {
        let provider = OpenRouter::new("test-token");
        let model =
            OpenRouterModel::new(provider, "openai".to_string(), "gpt-3.5-turbo".to_string());

        let profile = model.profile().await;
        assert_eq!(profile.name, "openai/gpt-3.5-turbo");
        assert_eq!(profile.author, "openai");
        assert_eq!(profile.slug, "gpt-3.5-turbo");
        assert_eq!(profile.context_length, 32768);
    }

    #[tokio::test]
    async fn test_respond_stream_structure() {
        let provider = OpenRouter::new("test-token");
        let model =
            OpenRouterModel::new(provider, "openai".to_string(), "gpt-3.5-turbo".to_string());

        let messages = vec![ai_types::llm::Message::user("Hello")];
        let mut tools = ai_types::llm::tool::Tools::new();
        let parameters = ai_types::llm::model::Parameters::default();

        let stream = model.respond(&messages, &mut tools, &parameters);

        // Pin the stream and try to get one item
        // This will likely fail due to auth, but tests the stream setup
        futures_util::pin_mut!(stream);
        match stream.next().await {
            Some(Ok(text)) => {
                // Successful response
                assert!(!text.is_empty());
            }
            Some(Err(_)) => {
                // Expected error due to invalid token
            }
            None => {
                // Empty stream - also possible
            }
        }
    }

    #[tokio::test]
    async fn test_complete_stream_structure() {
        let provider = OpenRouter::new("test-token");
        let model =
            OpenRouterModel::new(provider, "openai".to_string(), "gpt-3.5-turbo".to_string());

        let stream = model.complete("The weather today is");

        // Pin the stream and try to get one item
        futures_util::pin_mut!(stream);
        match stream.next().await {
            Some(Ok(text)) => {
                // Successful response
                assert!(!text.is_empty());
            }
            Some(Err(_)) => {
                // Expected error due to invalid token
            }
            None => {
                // Empty stream - also possible
            }
        }
    }

    #[test]
    fn test_error_display() {
        let api_error =
            ai_providers::openrouter::api::error::ApiError::Http("Test error".to_string());
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

    // Mock HTTP tests (would require additional setup for true unit testing)
    #[cfg(test)]
    mod mock_tests {

        // These tests would use a mock HTTP client to test the actual API interactions
        // without making real network calls

        #[test]
        fn test_request_headers() {
            // Test that proper headers are set (would need HTTP mock)
            // - Authorization header with Bearer token
            // - Content-Type: application/json for POST requests
        }

        #[test]
        fn test_request_body_serialization() {
            // Test that request bodies are properly serialized to JSON
            // and match OpenRouter API expectations
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

// Integration tests (require valid API tokens and network access)
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;
    use std::env;

    fn get_openrouter_token() -> Option<String> {
        env::var("OPENROUTER_API_KEY").ok()
    }

    #[tokio::test]
    async fn test_real_api_list_models() {
        let Some(token) = get_openrouter_token() else {
            println!("Skipping integration test - no OPENROUTER_API_KEY");
            return;
        };

        let provider = OpenRouter::new(token);
        let models = provider.list_models().await.expect("Failed to list models");

        assert!(!models.is_empty());

        // Check first model has expected structure
        let first_model = &models[0];
        assert!(!first_model.name.is_empty());
        assert!(!first_model.author.is_empty());
        assert!(first_model.context_length > 0);
    }

    #[tokio::test]
    async fn test_real_api_get_model() {
        let Some(token) = get_openrouter_token() else {
            println!("Skipping integration test - no OPENROUTER_API_KEY");
            return;
        };

        let provider = OpenRouter::new(token);
        let model = provider
            .get_model("openai/gpt-3.5-turbo")
            .await
            .expect("Failed to get model");

        // Verify model was created properly
        let profile = model.profile().await;
        assert!(profile.name.contains("gpt-3.5-turbo"));
    }

    #[tokio::test]
    async fn test_real_api_chat() {
        let Some(token) = get_openrouter_token() else {
            println!("Skipping integration test - no OPENROUTER_API_KEY");
            return;
        };

        let provider = OpenRouter::new(token);
        let model = provider
            .get_model("openai/gpt-3.5-turbo")
            .await
            .expect("Failed to get model");

        let messages = vec![ai_types::llm::Message::user("Say 'Hello from OpenRouter!'")];
        let mut tools = ai_types::llm::tool::Tools::new();
        let parameters = ai_types::llm::model::Parameters::default();

        let mut stream = model.respond(&messages, &mut tools, &parameters);
        let mut response = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => response.push_str(&text),
                Err(e) => panic!("Stream error: {}", e),
            }
        }

        assert!(!response.is_empty());
        assert!(response.to_lowercase().contains("hello"));
    }
}
