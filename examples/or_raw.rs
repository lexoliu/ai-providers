use ai_providers::openrouter::OpenRouter;
use ai_providers::openrouter::api::chat_completion::{ChatCompletionRequest, Message, Role};
use serde_json::Value;
use zenwave::Method;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let key = args.next().unwrap_or_default();
    let model = args
        .next()
        .unwrap_or_else(|| "deepseek/deepseek-chat-v3.1".to_string());
    let prompt = args.next().unwrap_or_else(|| "Hello!".to_string());

    let provider = OpenRouter::new(key);

    let req = ChatCompletionRequest {
        model: model.clone(),
        messages: vec![Message {
            role: Role::User,
            content: prompt.clone(),
        }],
        parameters: ai_providers::openrouter::api::types::Parameters::default(),
        ..Default::default()
    };

    let body = serde_json::to_string(&req).unwrap();
    eprintln!("Request JSON: {}", body);

    match provider
        .request::<_, Value>(Method::POST, "/chat/completions", req)
        .await
    {
        Ok(v) => {
            println!("JSON: {}", v);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
