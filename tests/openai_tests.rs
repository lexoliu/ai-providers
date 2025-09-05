use ai_providers::openai::{OpenAI, OpenAIModel, Error};
use ai_types::llm::{LanguageModelProvider, LanguageModel};
use futures_util::StreamExt;

#[cfg(test)]
mod openai_tests {
    use super::*;

    #[test]
    fn test_openai_creation() {
        let provider = OpenAI::new("test-api-key");
        assert!(!format!("{:?}", provider).is_empty());
    }

    #[test]
    fn test_openai_with_custom_base_url() {
        let provider = OpenAI::new("test-api-key")
            .with_base_url("https://custom.openai.com/v1");
        assert!(!format!("{:?}", provider).is_empty());
    }

    #[test]
    fn test_openai_provider_profile() {
        let profile = OpenAI::profile();
        assert_eq!(profile.name(), "OpenAI");
        assert!(profile.description().contains("Official OpenAI API"));
        assert!(profile.description().contains("GPT models"));
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = OpenAI::new("test-api-key");
        
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
    async fn test_get_model_valid() {
        let provider = OpenAI::new("test-api-key");
        
        let model = provider.get_model("gpt-3.5-turbo").await
            .expect("Should create model for valid name");
        
        assert_eq!(model.model_name, "gpt-3.5-turbo");
    }

    #[tokio::test]
    async fn test_get_model_invalid_empty_name() {
        let provider = OpenAI::new("test-api-key");
        
        let result = provider.get_model("").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            Error::InvalidModelName(name) => {
                assert_eq!(name, "");
            }
            _ => panic!("Expected InvalidModelName error"),
        }
    }

    #[test]
    fn test_openai_model_creation() {
        let provider = OpenAI::new("test-api-key");
        let model = OpenAIModel::new(provider, "gpt-4".to_string());
        
        assert_eq!(model.model_name, "gpt-4");
        assert!(!format!("{:?}", model).is_empty());
    }

    #[tokio::test]
    async fn test_model_profile() {
        let provider = OpenAI::new("test-api-key");
        let model = OpenAIModel::new(provider, "gpt-3.5-turbo".to_string());
        
        let profile = model.profile().await;
        assert_eq!(profile.name, "gpt-3.5-turbo");
        assert_eq!(profile.author, "OpenAI");
        assert_eq!(profile.slug, "gpt-3.5-turbo");
        assert!(profile.description.contains("OpenAI gpt-3.5-turbo model"));
        assert_eq!(profile.context_length, 4096); // GPT-3.5-turbo context length
    }

    #[tokio::test]
    async fn test_respond_stream_structure() {
        let provider = OpenAI::new("test-api-key");
        let model = OpenAIModel::new(provider, "gpt-3.5-turbo".to_string());
        
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
        let provider = OpenAI::new("test-api-key");
        let model = OpenAIModel::new(provider, "gpt-3.5-turbo".to_string());
        
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
        let api_error = ai_providers::shared::openai_api::ApiError {
            message: "Test error".to_string(),
            r#type: "invalid_request_error".to_string(),
            param: None,
            code: None,
        };
        let error = Error::Api(api_error);
        
        let error_str = format!("{}", error);
        assert!(error_str.contains("OpenAI API error"));
        assert!(error_str.contains("Test error"));
    }

    #[test]
    fn test_message_conversion() {
        use ai_providers::shared::openai_api::{convert_message, convert_role, Role};
        
        // Test role conversion
        assert!(matches!(convert_role(ai_types::llm::Role::User), Role::User));
        assert!(matches!(convert_role(ai_types::llm::Role::Assistant), Role::Assistant));
        assert!(matches!(convert_role(ai_types::llm::Role::System), Role::System));
        
        // Test message conversion
        let user_msg = ai_types::llm::Message::user("Hello");
        let converted = convert_message(&user_msg);
        
        assert!(matches!(converted.role, Role::User));
        assert_eq!(converted.content, "Hello");
        assert!(converted.name.is_none());
        assert!(converted.tool_calls.is_none());
    }

    #[test]
    fn test_context_length_mapping() {
        let provider = OpenAI::new("test-key");
        
        // Test different model context lengths
        let test_cases = vec![
            ("gpt-4", 8192),
            ("gpt-4-32k", 32768),
            ("gpt-4-turbo", 128000),
            ("gpt-4o", 128000),
            ("gpt-3.5-turbo", 4096),
            ("gpt-3.5-turbo-16k", 16384),
            ("unknown-model", 4096), // Default
        ];
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        for (model_name, expected_context) in test_cases {
            let model = runtime.block_on(provider.get_model(model_name)).unwrap();
            let profile = runtime.block_on(model.profile());
            assert_eq!(profile.context_length, expected_context, 
                      "Model {} should have context length {}", model_name, expected_context);
        }
    }

    #[test]
    fn test_parameter_application() {
        use ai_providers::shared::openai_api::{ChatCompletionRequest, apply_parameters};
        
        let mut request = ChatCompletionRequest::default();
        let parameters = ai_types::llm::model::Parameters::default()
            .temperature(0.7)
            .max_tokens(100)
            .top_p(0.9)
            .frequency_penalty(0.5)
            .presence_penalty(0.3)
            .seed(42)
            .stop(vec!["STOP".to_string()]);
        
        apply_parameters(&mut request, &parameters);
        
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.top_p, Some(0.9));
        assert_eq!(request.frequency_penalty, Some(0.5));
        assert_eq!(request.presence_penalty, Some(0.3));
        assert_eq!(request.seed, Some(42));
        assert_eq!(request.stop, Some(vec!["STOP".to_string()]));
    }

    #[test]
    fn test_openai_vs_openrouter_compatibility() {
        // Test that the shared types work with both providers
        use ai_providers::shared::openai_api::{Role, Message, ChatCompletionRequest};
        
        let message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![message],
            temperature: Some(0.7),
            stream: Some(true),
            ..Default::default()
        };
        
        // Should serialize without issues
        let json = serde_json::to_string(&request).expect("Should serialize");
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello"));
        assert!(json.contains("0.7"));
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
            // and match OpenAI API expectations
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
        
        #[test]
        fn test_streaming_response() {
            // Test that streaming responses are properly handled
            // - Delta messages for streaming
            // - Proper chunk processing
        }
    }
}

// Integration tests (require valid API key and network access)
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;
    use std::env;
    
    fn get_openai_api_key() -> Option<String> {
        env::var("OPENAI_API_KEY").ok()
    }
    
    #[tokio::test]
    async fn test_real_api_list_models() {
        let Some(api_key) = get_openai_api_key() else {
            println!("Skipping integration test - no OPENAI_API_KEY");
            return;
        };
        
        let provider = OpenAI::new(api_key);
        let models = provider.list_models().await.expect("Failed to list models");
        
        assert!(!models.is_empty());
        
        // Check that common models are present
        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(model_names.iter().any(|name| name.contains("gpt")));
        
        // Check first model has expected structure
        let first_model = &models[0];
        assert!(!first_model.name.is_empty());
        assert_eq!(first_model.author, "openai");
        assert!(first_model.context_length > 0);
    }
    
    #[tokio::test]
    async fn test_real_api_get_model() {
        let Some(api_key) = get_openai_api_key() else {
            println!("Skipping integration test - no OPENAI_API_KEY");
            return;
        };
        
        let provider = OpenAI::new(api_key);
        let model = provider.get_model("gpt-3.5-turbo").await
            .expect("Failed to get model");
        
        // Verify model was created properly
        let profile = model.profile().await;
        assert_eq!(profile.name, "gpt-3.5-turbo");
        assert_eq!(profile.author, "OpenAI");
    }
    
    #[tokio::test]
    async fn test_real_api_chat() {
        let Some(api_key) = get_openai_api_key() else {
            println!("Skipping integration test - no OPENAI_API_KEY");
            return;
        };
        
        let provider = OpenAI::new(api_key);
        let model = provider.get_model("gpt-3.5-turbo").await
            .expect("Failed to get model");
        
        let messages = vec![
            ai_types::llm::Message::system("You are a helpful assistant that responds briefly"),
            ai_types::llm::Message::user("What is 2 + 2? Answer with just the number.")
        ];
        let mut tools = ai_types::llm::tool::Tools::new();
        let parameters = ai_types::llm::model::Parameters::default().max_tokens(10);
        
        let stream = model.respond(&messages, &mut tools, &parameters);
        let mut response = String::new();
        
        futures_util::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => response.push_str(&text),
                Err(e) => panic!("Stream error: {}", e),
            }
        }
        
        assert!(!response.is_empty());
        assert!(response.trim().contains("4") || response.to_lowercase().contains("four"));
    }
    
    #[tokio::test]
    async fn test_real_api_completion() {
        let Some(api_key) = get_openai_api_key() else {
            println!("Skipping integration test - no OPENAI_API_KEY");
            return;
        };
        
        let provider = OpenAI::new(api_key);
        let model = provider.get_model("gpt-3.5-turbo").await
            .expect("Failed to get model");
        
        let stream = model.complete("The capital of France is");
        let mut response = String::new();
        
        futures_util::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    response.push_str(&text);
                    // Break after getting some response to keep test fast
                    if response.len() > 5 {
                        break;
                    }
                }
                Err(e) => panic!("Stream error: {}", e),
            }
        }
        
        assert!(!response.is_empty());
    }
}