use ai_providers::openrouter::OpenRouter;
use ai_types::llm::LanguageModelProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::args().nth(1).unwrap_or_default();
    let provider = OpenRouter::new(key);

    match provider.list_models().await {
        Ok(models) => {
            println!("Found {} models. Showing first 20:", models.len());
            for m in models.iter().take(20) {
                println!("- {} (author: {}, slug: {}, ctx: {})",
                    m.name, m.author, m.slug, m.context_length);
            }
        }
        Err(e) => {
            eprintln!("Failed to list models: {}", e);
        }
    }

    Ok(())
}

