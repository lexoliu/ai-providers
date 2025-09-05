use ai_providers::openrouter::OpenRouter;
use ai_types::{
    LanguageModel,
    llm::{LanguageModelProvider, Message, model::Parameters, tool::Tools},
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let key = args.next().unwrap_or_default();
    let model_name = args
        .next()
        .unwrap_or_else(|| "deepseek/deepseek-chat-v3.1".to_string());
    let prompt = args.next().unwrap_or_else(|| "Hello!".to_string());

    let provider = OpenRouter::new(key);
    let model = provider.get_model(&model_name).await?;

    let mut tools = Tools::new();
    let params = Parameters::default().temperature(0.7).max_tokens(200);
    let msgs = vec![Message::user(prompt)];
    let stream = model.respond(&msgs, &mut tools, &params);
    futures_util::pin_mut!(stream);

    while let Some(part) = stream.next().await {
        match part {
            Ok(text) => print!("{}", text),
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }
    println!();
    Ok(())
}
