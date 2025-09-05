# AI Providers Chatbot Example

This example demonstrates how to build a CLI chatbot using the `ai-providers` crate that can work with multiple AI providers.

## Features

- 🤖 Support for multiple AI providers (OpenAI, OpenRouter, Anthropic)
- 💬 Interactive conversation with persistent chat history
- 🔄 Streaming responses for real-time interaction
- 🎯 Simple command-line interface
- 🧪 Comprehensive test coverage

## Usage

### Prerequisites

You'll need an API key for the provider you want to use:

- **OpenAI**: Get your API key from [OpenAI Platform](https://platform.openai.com/api-keys)
- **OpenRouter**: Get your API key from [OpenRouter](https://openrouter.ai/keys)
- **Anthropic**: Get your API key from [Anthropic Console](https://console.anthropic.com/)

### Running the Chatbot

```bash
# Using OpenAI
cargo run --example chatbot openai YOUR_OPENAI_API_KEY

# Using OpenRouter
cargo run --example chatbot openrouter YOUR_OPENROUTER_API_KEY

# Using Anthropic
cargo run --example chatbot anthropic YOUR_ANTHROPIC_API_KEY
```

### Commands

Once the chatbot is running, you can use these commands:

- **chat normally**: Just type your message and press Enter
- **`clear`**: Clear the conversation history and start fresh
- **`quit`** or **`exit`**: End the conversation and close the program

### Example Session

```
🤖 AI Chatbot - gpt-3.5-turbo (OpenAI)
💭 Context Length: 4096 tokens
📝 OpenAI gpt-3.5-turbo model
───────────────────────────────────────────
Type 'quit' or 'exit' to end the conversation
Type 'clear' to start a new conversation
───────────────────────────────────────────

💬 You: Hello! Can you help me understand Rust ownership?

🤖 gpt-3.5-turbo: Of course! Rust's ownership system is one of its most distinctive features. It's designed to manage memory safely without needing a garbage collector...

💬 You: Can you give me a simple example?

🤖 gpt-3.5-turbo: Sure! Here's a simple example of ownership in Rust:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2
    
    // println!("{}", s1); // This would cause a compile error!
    println!("{}", s2); // This works fine
}
```

💬 You: clear
🗑️  Conversation cleared!

💬 You: quit
👋 Goodbye!
```

## Code Structure

The example demonstrates several key concepts:

### Provider Abstraction

The `ChatBot` enum handles different AI providers uniformly:

```rust
enum ChatBot {
    OpenAI { model: OpenAIModel, profile: Profile },
    OpenRouter { model: OpenRouterModel, profile: Profile },
    Anthropic { model: AnthropicModel, profile: Profile },
}
```

### Streaming Responses

Real-time streaming is handled for all providers:

```rust
async fn respond(&self, messages: &[Message]) -> Result<String, Box<dyn std::error::Error>> {
    // Handle streaming for each provider type
    match self {
        ChatBot::OpenAI { model, .. } => {
            let stream = model.respond(messages, &mut tools, &parameters);
            // Process stream chunks...
        }
        // ... other providers
    }
}
```

### Conversation Management

The chatbot maintains conversation history and handles special commands:

```rust
let mut conversation: Vec<Message> = vec![];

// Add user message
conversation.push(Message::user(input));

// Get and display response
match chatbot.respond(&conversation).await {
    Ok(response) => {
        conversation.push(Message::assistant(&response));
    }
    Err(e) => eprintln!("❌ Error: {}", e),
}
```

## Testing

The example includes comprehensive tests:

```bash
# Run the example tests
cargo test --example chatbot

# Build the example
cargo build --example chatbot
```

Test coverage includes:
- Chatbot creation for all providers
- Profile validation
- Error handling for network failures

## Customization

You can easily customize the chatbot:

### Change Default Models

Modify the model selection in `main()`:

```rust
let chatbot = match provider_name.as_str() {
    "openai" => ChatBot::new_openai(api_key, "gpt-4").await?, // Use GPT-4
    "openrouter" => ChatBot::new_openrouter(api_key, "anthropic/claude-3-opus-20240229").await?,
    "anthropic" => ChatBot::new_anthropic(api_key, "claude-3-opus-20240229").await?, // Use Claude Opus
    // ...
};
```

### Adjust Parameters

Modify the response parameters in the `respond()` method:

```rust
let parameters = Parameters::default()
    .temperature(0.9)        // More creative responses
    .max_tokens(2000)        // Longer responses
    .top_p(0.9);             // Nucleus sampling
```

### Add More Commands

Extend the command handling logic:

```rust
match input.to_lowercase().as_str() {
    "quit" | "exit" => break,
    "clear" => { /* clear logic */ }
    "help" => {
        println!("Available commands: clear, quit, exit, help");
        continue;
    }
    "save" => {
        // Save conversation to file
        continue;
    }
    _ => {} // Regular chat message
}
```

## Error Handling

The example includes robust error handling:

- **Network errors**: Graceful handling of API failures
- **Invalid API keys**: Clear error messages
- **Rate limiting**: Automatic error display
- **Malformed responses**: Safe parsing with error recovery

## Integration with Your Project

To integrate this chatbot pattern into your own project:

1. Add the dependencies to your `Cargo.toml`:
```toml
[dependencies]
ai-providers = { path = "../path/to/ai-providers" }
ai-types = { path = "../path/to/ai-types" }
tokio = { version = "1.0", features = ["full"] }
futures-util = "0.3"
```

2. Use the `ChatBot` pattern as a template
3. Customize the providers and models for your use case
4. Add your own UI layer (CLI, web, GUI, etc.)

## Contributing

This example is part of the `ai-providers` crate. To contribute:

1. Make your changes to `examples/chatbot.rs`
2. Run tests: `cargo test --example chatbot`
3. Ensure the example builds: `cargo build --example chatbot`
4. Test with actual API keys (optional but recommended)

## License

This example is part of the `ai-providers` crate and follows the same license.