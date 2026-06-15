# Ollama Provider

Ollama allows you to run large language models locally. This is a great option if you want to keep your code private and avoid sending data to external APIs.

## Requirements

1. Install [Ollama](https://ollama.com/).
2. Pull a model. We recommend `qwen2.5-coder:3b` for fast and accurate code summarization.
   ```bash
   ollama pull qwen2.5-coder:3b
   ```
3. Ensure the Ollama server is running (usually it runs automatically in the background after installation).

## Configuration

Update your `asum.toml` file to use Ollama:

```toml
[general]
active_provider = "ollama"

[ollama]
# The name of the model you pulled
model = "qwen2.5-coder:3b"
# The API endpoint for Ollama
url = "http://localhost:11434/api/chat"
```

## Custom URL
If your Ollama instance is hosted on another machine, or behind a reverse proxy, you can modify the `url` field to point to your custom endpoint.
