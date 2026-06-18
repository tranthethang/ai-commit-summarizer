# Groq Provider

Groq provides extremely fast inference using LPU (Language Processing Unit) technology for models like Llama 3.

## Requirements

1. Obtain a Groq API key from the [Groq Console](https://console.groq.com/keys).
2. Choose a model. We recommend `llama-3.3-70b-versatile` or `llama3-8b-8192` for general use.

## Configuration

Update your `asum.toml` file to use Groq:

```toml
[general]
active_provider = "groq"

[groq]
api_key = "gsk_..."
model = "llama-3.3-70b-versatile"
```

## Custom URL

If you need to route your requests through an enterprise proxy or use a different endpoint, you can optionally provide a `url` field:

```toml
[groq]
api_key = "gsk_..."
model = "llama-3.3-70b-versatile"
url = "https://api.groq.com/openai/v1/chat/completions"
```
