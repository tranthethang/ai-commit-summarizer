# asum (AI Commit Summarizer)

**asum** is a lightweight CLI tool written in Rust that automatically generates professional git commit messages using AI models. It helps developers maintain a clean and consistent commit history without the manual effort of summarizing changes.

---

## Features

- **Multi-Backend Support**: Supports both local [Ollama](https://ollama.com/) models and [Google Gemini API](https://ai.google.dev/).
- **Strategy Pattern**: Modular architecture allows for easy extension to other AI providers.
- **Smart Filtering**: Automatically filters `git diff` to focus on relevant source code while ignoring lock files and binaries.
- **Clipboard Integration**: Automatically copies the generated commit message to your system clipboard.
- **Persistent Configuration**: Stores settings in a local SQLite database (`asum.db`).

---

## Requirements

Before installing, ensure you have the following tools set up:

1. **Rust & Cargo**: [Install Rust](https://www.rust-lang.org/tools/install)
2. **AI Provider**:
   - **Ollama**: [Download Ollama](https://ollama.com/) and pull a model (e.g., `ollama pull llama3.2:1b`).
   - **Gemini**: Obtain an API key from [Google AI Studio](https://aistudio.google.com/).

---

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/thangtt/ai-commit-summarizer.git
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

Settings are stored in `asum.db`. On the first run, the tool initializes the database with default values.

### Default Config
- `active_provider`: `ollama`
- `ollama_url`: `http://localhost:11434/api/generate`
- `ollama_model`: `llama3.2:1b`
- `gemini_model`: `gemini-1.5-flash`

### Switching to Gemini
To use Gemini, you need to update the database:
```bash
sqlite3 asum.db "UPDATE config SET value = 'gemini' WHERE key = 'active_provider';"
sqlite3 asum.db "UPDATE config SET value = 'YOUR_GEMINI_API_KEY' WHERE key = 'gemini_api_key';"
```

---

## Uninstallation

To remove the tool from your system:

```bash
chmod +x uninstall.sh
./uninstall.sh
```
