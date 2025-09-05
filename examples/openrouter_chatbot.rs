//! OpenRouter CLI Chatbot Example
//!
//! - Prompts for API key and model slug on start
//! - Streams responses and maintains conversation context
//! - Commands: type `exit` or `quit` to leave, `clear` to reset history

use ai_providers::openrouter::OpenRouter;
use ai_types::{
    LanguageModel,
    llm::{LanguageModelProvider, Message, model::Parameters, tool::Tools},
};
use futures_util::StreamExt;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prompt for API key
    let api_key = prompt("Enter OpenRouter API key: ")?;
    if api_key.trim().is_empty() {
        eprintln!("API key is required.");
        return Ok(());
    }

    // Prompt for model slug
    let model_name = prompt_with_default(
        "Enter model (e.g. deepseek/deepseek-chat-v3.1:free): ",
        "deepseek/deepseek-chat-v3.1:free",
    )?;

    // Init provider and model
    let provider = OpenRouter::new(api_key.trim());
    let model = match provider.get_model(model_name.trim()).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to create model: {}", e);
            return Ok(());
        }
    };
    let profile = model.profile().await;

    println!("🤖 OpenRouter Chatbot - {} ({})", profile.name, profile.author);
    println!("💭 Context Length: {} tokens", profile.context_length);
    println!("───────────────────────────────────────────");
    println!("Type 'quit' or 'exit' to end the conversation");
    println!("Type 'clear' to start a new conversation");
    println!("───────────────────────────────────────────");

    let mut conversation: Vec<Message> = vec![];
    let mut tools = Tools::new();
    let parameters = Parameters::default().temperature(0.7).max_tokens(800);

    loop {
        print!("\n💬 You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            "clear" => {
                conversation.clear();
                println!("🗑️  Conversation cleared!");
                continue;
            }
            _ => {}
        }

        // Add user message
        conversation.push(Message::user(input));

        // Stream response
        print!("🤖 {}: ", profile.name);
        io::stdout().flush()?;

        let mut response = String::new();
        {
            let stream = model.respond(&conversation, &mut tools, &parameters);
            futures_util::pin_mut!(stream);

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        print!("{}", text);
                        io::stdout().flush().ok();
                        response.push_str(&text);
                    }
                    Err(e) => {
                        eprintln!("\n❌ Error: {}", e);
                        break;
                    }
                }
            }
        }

        if !response.is_empty() {
            conversation.push(Message::assistant(&response));
        }
    }

    Ok(())
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{}", label);
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim_end().to_string())
}

fn prompt_with_default(label: &str, default: &str) -> io::Result<String> {
    print!("{}[default: {}] ", label, default);
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let input = s.trim();
    if input.is_empty() { Ok(default.to_string()) } else { Ok(input.to_string()) }
}
