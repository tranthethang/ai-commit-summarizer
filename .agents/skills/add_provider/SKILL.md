---
name: add_provider
description: Guides the implementation and integration of a new AI provider into the asum project.
---

# Add AI Provider

Use this skill when you need to integrate a new AI provider into the `asum` system. This skill ensures that we gather all API requirements, follow the established codebase architecture, write comprehensive tests, and adhere to project standards.

## Inputs
- **Provider Name**: The name of the AI provider (e.g., `DeepSeek`, `Anthropic`).
- **Documentation URL**: The main URL for the AI provider's API reference.
- **Model Name(s)**: Default model names to configure.

## Workflow

### 1. Research and Documentation Review
Before coding, perform thorough research to understand the provider's API specifications:
- Read documentation at the provided URL.
- Identify the authentication method (e.g., API key in bearer token, custom headers).
- Determine request payload structure (JSON body, required fields, system prompt fields).
- Identify response payload structure and how to parse the generated text.
- Check rate limiting policies and error response status codes (specifically look for HTTP 429).

### 2. Plan Integration
Create a detailed implementation plan referencing [adding-providers.md](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/docs/adding-providers.md). The plan must cover:
- Config structure changes in [config.rs](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/src/config.rs).
- Provider implementation file: `src/summarizer/[provider_name].rs`.
- Registration in [mod.rs](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/src/summarizer/mod.rs).
- Configuration additions in [asum.toml](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml) and [asum.toml.example](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml.example).
- Testing strategies using mock servers.

### 3. Implement the Provider
Strictly follow the 5 steps in [adding-providers.md](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/docs/adding-providers.md).

#### Coding and Naming Conventions
- **Naming Conventions**: Use `PascalCase` for structs/enums/traits and `snake_case` for variables/functions/modules.
- **Safety**: Do not use `unwrap()` or `expect()`. Handle all errors via `Result` and propagate using `?`.
- **File Length Limit**: No source file in `src/` should exceed 300 lines of code. Split complex logic or implementations into sub-modules if close to the limit.
- **Request Retry**: Implement exponential backoff for requests, specifically handling HTTP 429 rate limits.

### 4. Write and Run Tests
- **No Tests in src**: Unit and integration tests (including `#[cfg(test)] mod tests`) must never be placed inside `./src`.
- Create a new integration test file under `tests/` (e.g., `tests/[provider_name]_test.rs`).
- Use mock servers or mock responses to test:
  - Valid and invalid credentials/configurations.
  - Successful responses and error fallbacks.
  - Rate limiting (exponential backoff behavior).
- Update config tests in `tests/config_test.rs`.
- Run formatting and analysis:
  ```bash
  cargo fmt
  cargo clippy --all-targets
  cargo test
  ```

### 5. Finalize Documentation
- Write all comments, documentation, and config guides in English only.
- Do not use emojis in any Markdown files.
- Document configuration settings in [asum.toml](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml) and [asum.toml.example](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml.example).
