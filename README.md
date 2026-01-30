# asum (AI Commit Summarizer)

**asum** is a lightweight CLI tool written in Rust that automatically generates professional git commit messages using AI models. It helps developers maintain a clean and consistent commit history without the manual effort of summarizing changes.

---

## Features

- **Multi-Backend Support**: Supports both local [Ollama](https://ollama.com/) models and [Google Gemini API](https://ai.google.dev/).
- **Strategy Pattern**: Modular architecture allows for easy extension to other AI providers.
- **Smart Filtering**: Automatically filters `git diff` to focus on relevant source code while ignoring lock files and binaries.
- **Clipboard Integration**: Automatically copies the generated commit message to your system clipboard.
- **Flexible Configuration**: Supports local and global `asum.toml` configuration files.

---

## Requirements

Before installing, ensure you have the following tools set up:

1. **Rust & Cargo**: [Install Rust](https://www.rust-lang.org/tools/install)
2. **AI Provider**:
   - **Ollama**: [Download Ollama](https://ollama.com/) and pull a model (e.g., `ollama pull qwen2.5-coder:3b`).
   - **Gemini**: Obtain an API key from [Google AI Studio](https://aistudio.google.com/).

---

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/tranthethang/ai-commit-summarizer.git
   cd ai-commit-summarizer
   ```

2. **Run the installer**:
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
   *Note: The installer will compile the project in release mode and move the binary to `/usr/local/bin`.*

---

## Usage

Simply stage your changes and run `asum`:

```bash
git add .
asum
```

![Usage](./screenshot.png)

The tool will analyze your staged changes, display a suggested commit message, and copy it to your clipboard. You can then simply press `Cmd+V` (or `Ctrl+V`) to paste it into your `git commit` command.

---

## Configuration

**asum** loads configuration from a file named `asum.toml`. It searches for this file in the following order:

1.  **Local**: The current directory where you run the `asum` command.
2.  **Global**: Your user home directory at `~/.asum/asum.toml`.

If neither is found, the tool will report an error.

### Example Configuration

You can use [asum.toml.example](./asum.toml.example) as a template:

```toml
[general]
active_provider = "ollama"
max_diff_length = 36000

[ai_params]
num_predict = 250
temperature = 0.1
top_p = 0.9

[gemini]
api_key = "YOUR_GEMINI_API_KEY"
model = "gemini-2.0-flash"

[ollama]
model = "qwen2.5-coder:3b"
url = "http://localhost:11434/api/generate"
```

### Verification

You can verify the syntax of your `asum.toml` file by running:

```bash
asum verify
```

---

## Uninstallation

To remove the tool from your system:

```bash
chmod +x uninstall.sh
./uninstall.sh
```
