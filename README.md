# asum (AI Commit Summarizer)

![AI Commit Summarizer](assets/repo.png)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellowgreen.svg)](https://conventionalcommits.org)

**asum** is a lightweight, blazing-fast CLI tool written in Rust that automatically generates professional git commit messages using AI models. It helps developers maintain a clean, consistent, and meaningful commit history effortlessly by adhering to the **Conventional Commits 1.0.0** specification.

---

## Quick Start: Installation & Uninstallation

Get up and running in seconds. No complex setup or compilation required.

### One-Command Install
Detects your OS and architecture, downloads the latest pre-compiled binary, and adds it to your path automatically:

```bash
curl -fsSL https://raw.githubusercontent.com/tranthethang/ai-commit-summarizer/main/install.sh | bash
```

### One-Command Uninstall
Completely removes `asum` and its configurations from your system:

```bash
curl -fsSL https://raw.githubusercontent.com/tranthethang/ai-commit-summarizer/main/uninstall.sh | bash
```

<details>
<summary>Build from Source (Alternative)</summary>

If you prefer building from source using Rust:
```bash
git clone https://github.com/tranthethang/ai-commit-summarizer.git
cd ai-commit-summarizer
cargo build --release
sudo cp target/release/asum /usr/local/bin/
```
</details>

---

## Features

- **Strict Conventional Commits**: Generates standardized headers matching `<type>(<scope>): <description>` and detailed bodies.
- **Multi-Provider Support**: Integrate with Google Gemini, local Ollama models, OpenAI, or Google Vertex AI.
- **Intelligent File List Fallback**: If you stage changes that are not source code (e.g. assets, lock files, binary files, or ignored extensions), `asum` automatically falls back to summarizing based on the list of staged filenames so you never get empty diff errors.
- **Context-Aware System Prompts**: Employs Few-Shot Prompting and precise system instructions to guarantee high-quality, concise, and structured commit proposals.
- **Smart Diff Filtering**: Focuses only on relevant source code files, ignoring noise like lock files, large generated assets, or binaries.
- **Clipboard Integration**: Copies the final message to your system clipboard automatically so you can paste it immediately.

---

## Supported Providers

**asum** supports a wide range of cloud and local AI backends. Click a provider for its dedicated configuration details:

| Provider | Default Model | Configuration Guide | Use Case |
| :--- | :--- | :--- | :--- |
| **Google Gemini** | `gemini-2.0-flash` | [Gemini Setup Guide](./docs/providers/gemini.md) | High quality, low latency, free/pay-as-you-go. |
| **OpenAI** | `gpt-4o-mini` | [OpenAI Setup Guide](./docs/providers/openai.md) | Industry standard, OpenAI-compatible APIs (LM Studio, vLLM). |
| **Google Vertex AI** | `gemini-1.5-flash` | [Vertex AI Setup Guide](./docs/providers/vertexai.md) | Enterprise environments, auto-authenticates via `gcloud`. |
| **Ollama** | `qwen2.5-coder:3b` | [Ollama Setup Guide](./docs/providers/ollama.md) | 100% Local, private, and offline execution. |

---

## Usage

Simply stage your changes and run `asum` in your terminal:

```bash
# 1. Stage your changes
git add .

# 2. Generate and copy the commit message
asum
```

![Usage](./screenshot.png)

`asum` will analyze your staged diff (or file list), output the suggested commit message, and copy it to your system clipboard. Simply press `Cmd+V` (or `Ctrl+V`) to paste it into your `git commit` command.

---

## Configuration

**asum** loads its configuration from a file named `asum.toml`. It searches for this file in the following order:
1. **Local**: The current working directory (useful for project-specific models/settings).
2. **Global**: Your user home directory at `~/.asum/asum.toml`.

### Example `asum.toml`

Customize the AI settings, active provider, templates, and max diff lengths. Use [asum.toml.example](./asum.toml.example) as a template:

```toml
[general]
active_provider = "ollama"  # Options: "gemini", "ollama", "openai", "vertexai"
max_diff_length = 36000     # Limit diff size sent to AI

[prompts]
# Optional: Identity and rules for the AI
# system_prompt = "You are an expert Git Commit Generator..."
# Optional: Template for the user message. Use {{diff}} as placeholder.
# user_prompt = "[INPUT DIFF]\n{{diff}}\n\n[OUTPUT]"

[ai_params]
num_predict = 500
temperature = 0.1
top_p = 0.9

[gemini]
api_key = "YOUR_GEMINI_API_KEY"
model = "gemini-2.0-flash"

[ollama]
model = "qwen2.5-coder:3b"
url = "http://localhost:11434/api/chat"

[openai]
api_key = "YOUR_OPENAI_API_KEY"
model = "gpt-4o-mini"
# url = "https://api.openai.com/v1/chat/completions" # Optional custom url

[vertexai]
project_id = "YOUR_GCP_PROJECT_ID"
location = "us-central1"
model = "gemini-1.5-flash"
# access_token = "ya29..." # Optional static token. If empty, falls back to `gcloud`.
```

### Config Verification

Verify the syntax and completeness of your `asum.toml` by running:

```bash
asum verify
```

---

## Testing & Coverage

### Running Tests
To run the automated Rust tests:
```bash
cargo test
```

### Coverage Report
Generate a detailed HTML coverage report using `grcov`:
```bash
# Prerequisites
cargo install grcov
rustup component add llvm-tools-preview

# Run the coverage script
chmod +x coverage.sh
./coverage.sh
```
The resulting HTML report will be generated at `./coverage/index.html`.

---

## CI/CD and Automation

For details on how pre-compiled binaries are built and released via GitHub Actions, see the [Automation and Releases Documentation](./docs/automation-and-releases.md).
