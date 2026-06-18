# GitHub Models Provider

GitHub Models offers access to a variety of large language models, including GPT-4o mini, Meta Llama, and others directly via the GitHub API.

## Requirements

1. Obtain a GitHub Personal Access Token (PAT) with `models` access scope from [GitHub Developer Settings](https://github.com/settings/tokens).
2. Choose a model. We recommend `gpt-4o-mini` or other lightweight chat models for commit message generation.

## Configuration

Update your `asum.toml` file to use GitHub Models:

```toml
[general]
active_provider = "github"

[github]
api_key = "your-github-pat"
model = "gpt-4o-mini"
```

## Custom URL

If you need to route your requests through an enterprise proxy or use a custom endpoint, you can optionally provide a `url` field:

```toml
[github]
api_key = "your-github-pat"
model = "gpt-4o-mini"
url = "https://models.github.ai/inference/chat/completions"
```
