# Mistral AI Provider

Mistral AI offers powerful open-weights and commercial models.

## Requirements

1. Obtain a Mistral API key from the [Mistral Console](https://console.mistral.ai/).
2. Choose a model. We recommend `mistral-small-latest` or `open-mixtral-8x22b` for commit message generation.

## Configuration

Update your `asum.toml` file to use Mistral:

```toml
[general]
active_provider = "mistral"

[mistral]
api_key = "your-mistral-api-key"
model = "mistral-small-latest"
```

## Custom URL

If you need to route your requests through an enterprise proxy or use a different endpoint, you can optionally provide a `url` field:

```toml
[mistral]
api_key = "your-mistral-api-key"
model = "mistral-small-latest"
url = "https://api.mistral.ai/v1/chat/completions"
```
