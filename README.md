# asum (AI Commit Summarizer)

**asum** is a lightweight CLI tool written in Rust that automatically generates professional git commit messages using AI models. It helps developers maintain a clean and consistent commit history by following the **Conventional Commits 1.0.0** specification.

---

## Features

- **Conventional Commits 1.0.0**: Generates messages with strict `<type>(<scope>): <description>` headers and optional bodies.
- **Advanced Prompting**: Uses **Few-shot Prompting** and **System Instructions** to ensure high-quality and consistent output.
- **Multi-Backend Support**: Supports both local [Ollama](https://ollama.com/) (via Chat API) and [Google Gemini API](https://ai.google.dev/) (via System Instructions).
- **Strategy Pattern**: Modular architecture allows for easy extension to other AI providers.
- **Smart Filtering**: Automatically filters `git diff` to focus on relevant source code while ignoring lock files and binaries.
- **Clipboard Integration**: Automatically copies the generated commit message to your system clipboard.
- **Flexible Configuration**: Supports local and global `asum.toml` configuration files with separate system and user prompt templates.

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

### Example Configuration

You can use [asum.toml.example](./asum.toml.example) as a template:

```toml
[general]
active_provider = "ollama"
max_diff_length = 36000

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
```

### Verification

You can verify the syntax of your `asum.toml` file by running:

```bash
asum verify
```

---

## Testing & Coverage

**asum** includes a comprehensive suite of tests to ensure stability and correctness.

### Running Tests

To run the automated tests, use the standard cargo command:

```bash
cargo test
```

### Coverage Report

To generate a detailed HTML coverage report, you can use the provided `coverage.sh` script. This script uses `grcov` to aggregate coverage data.

**Prerequisites:**
- `grcov`: `cargo install grcov`
- `llvm-tools-preview`: `rustup component add llvm-tools-preview`

**Generate Report:**
```bash
chmod +x coverage.sh
./coverage.sh
```

After running the script, you can find the HTML report at `./coverage/index.html`.

---

## Uninstallation

To remove the tool from your system:

```bash
chmod +x uninstall.sh
./uninstall.sh
```
