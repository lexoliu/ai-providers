# ai-providers

Implementations of specific cloud provider models for the `ai-types` crate.

[![Crates.io](https://img.shields.io/crates/v/ai-providers.svg)](https://crates.io/crates/ai-providers)
[![Documentation](https://docs.rs/ai-providers/badge.svg)](https://docs.rs/ai-providers)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

The `ai-providers` crate provides concrete implementations of various AI models for different cloud providers, allowing seamless integration with the `ai-types` crate. This package enables developers to utilize models from OpenAI, Anthropic, and local implementations without changing their application logic.

## Features

- **Provider Implementations**: Specific models for OpenAI, Anthropic, and local environments.
- **Unified Interface**: All models adhere to the traits defined in the `ai-types` crate.
- **Easy Integration**: Switch between providers with minimal changes to your codebase.

## Getting Started

To use the `ai-providers` crate, add it to your `Cargo.toml`:

```toml
[dependencies]
ai-providers = "0.1.0"
ai-types = "0.0.1"
```

## Usage

### OpenAI Model

```rust
use ai_providers::OpenAIModel;
use ai_types::LanguageModel;

let model = OpenAIModel::new("your-api-key");
let response = model.generate("What is the capital of France?").await?;
```

### Anthropic Model

```rust
use ai_providers::AnthropicModel;
use ai_types::LanguageModel;

let model = AnthropicModel::new("your-api-key");
let response = model.generate("Tell me about the universe.").await?;
```

### Local Model

```rust
use ai_providers::LocalModel;
use ai_types::LanguageModel;

let model = LocalModel::new("path/to/local/model");
let response = model.generate("Generate a haiku.").await?;
```

## License

MIT License - see [LICENSE](LICENSE) for details.