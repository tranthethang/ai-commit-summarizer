# OpenAI Provider

OpenAI provides industry-leading models like `gpt-4o` and `gpt-4o-mini`.

## Requirements

1. Obtain an API key from the [OpenAI Developer Platform](https://platform.openai.com/api-keys).
2. Ensure you have sufficient credits or a linked billing account.

## Configuration

Update your `asum.toml` file to use OpenAI:

```toml
[general]
active_provider = "openai"

[openai]
api_key = "sk-..."
model = "gpt-4o-mini"
```

## Custom URL / Compatible Endpoints

Many other providers and local tools (like vLLM, LM Studio) expose an "OpenAI Compatible API". You can use the `openai` provider in `asum` to connect to them by overriding the `url`:

```toml
[openai]
api_key = "your-api-key-if-needed"
model = "your-model-name"
# Example for a local OpenAI-compatible server
url = "http://localhost:8000/v1/chat/completions"
```
