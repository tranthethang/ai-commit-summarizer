# Integrating a New AI Provider

This guide outlines the steps required to develop and integrate a new AI Provider into the AI Commit Summarizer (`asum`) project.

## Overview

The application utilizes a modular design for AI providers. The core logic is defined by the `Summarizer` trait in `src/summarizer/mod.rs`. To add a new provider, you will need to update the configuration module, implement the `Summarizer` trait for the new provider, register it in the provider factory, update template configurations, and add corresponding tests.

---

## Step 1: Update the Configuration System

You must define the configuration parameters for the new provider in [config.rs](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/src/config.rs).

1. **Add to the Provider Enum**:
   Add the new provider variant to the `Provider` enum. Use `#[serde(rename = "your_provider")]` to specify how it appears in `asum.toml`.
   ```rust
   pub enum Provider {
       // ... existing providers
       #[serde(rename = "your_provider")]
       YourProvider,
   }
   ```

2. **Add to the ProviderConfig Enum**:
   Define the variant under `ProviderConfig` to store the active provider settings:
   ```rust
   pub enum ProviderConfig {
       // ... existing configurations
       YourProvider {
           api_key: String,
           model: String,
           url: Option<String>,
       },
   }
   ```

3. **Define a TOML Parsing Struct**:
   Create a dedicated configuration struct that maps to the TOML section in `asum.toml`:
   ```rust
   #[derive(Debug, Deserialize, Serialize, Clone)]
   struct YourProviderConfig {
       pub api_key: String,
       pub model: String,
       pub url: Option<String>,
   }
   ```
   Then, add this struct as an optional field in the `TomlConfig` struct:
   ```rust
   struct TomlConfig {
       // ...
       pub your_provider: Option<YourProviderConfig>,
   }
   ```

4. **Map the Provider in Configuration Loader**:
   In `AsumConfig::load_from_toml_impl`, retrieve and build the provider config when active:
   ```rust
   let provider = match toml_config.general.active_provider {
       // ...
       Provider::YourProvider => {
           let config = toml_config
               .your_provider
               .as_ref()
               .ok_or_else(|| anyhow!("Missing [your_provider] configuration section"))?;
           ProviderConfig::YourProvider {
               api_key: config.api_key.clone(),
               model: config.model.clone(),
               url: config.url.clone(),
           }
       }
   };
   ```

5. **Implement Validation**:
   Create a validation function to verify configuration validity at startup:
   ```rust
   fn verify_your_provider_config(config: Option<&YourProviderConfig>) -> Result<()> {
       let cfg = config.ok_or_else(|| {
           anyhow::anyhow!("[your_provider] section is required when active_provider = \"your_provider\"")
       })?;
       if cfg.model.is_empty() {
           anyhow::bail!("model in [your_provider] section cannot be empty");
       }
       if cfg.api_key.is_empty() {
           anyhow::bail!("api_key in [your_provider] section cannot be empty");
       }
       Ok(())
   }
   ```
   Call this helper in the `verify_toml_impl` match block:
   ```rust
   match toml_config.general.active_provider {
       // ...
       Provider::YourProvider => verify_your_provider_config(toml_config.your_provider.as_ref()),
   }
   ```

---

## Step 2: Implement the Summarizer Trait

Create a new implementation file `src/summarizer/your_provider.rs` to handle request construction and parsing.

1. **Define the Provider Struct**:
   ```rust
   use crate::summarizer::{AIConfig, Summarizer};
   use async_trait::async_trait;
   use reqwest::Client;

   pub struct YourProviderProvider {
       pub config: AIConfig,
       client: Client,
       pub base_url: String,
   }
   ```

2. **Implement Initialization (`new`)**:
   Implement a `new` method to initialize your provider struct, reusing the shared HTTP client generator:
   ```rust
   impl YourProviderProvider {
       pub fn new(config: AIConfig) -> Self {
           let base_url = config
               .api_url
               .clone()
               .unwrap_or_else(|| "https://api.your-provider.com".to_string());
           Self {
               config,
               client: crate::summarizer::build_http_client(),
               base_url,
           }
       }
   }
   ```

3. **Implement the `Summarizer` Trait**:
   Implement `async fn summarize(&self, diff: &str) -> anyhow::Result<String>`:
   - Generate prompt payload using `generate_prompt`.
   - Setup authorization and request headers.
   - Execute HTTP call, handling rate limits (e.g., HTTP 429 status code) via an exponential backoff retry loop.
   - Check response status using `check_response_status`.
   - Parse raw response body and clean the summary output using `clean_ai_response`.

---

## Step 3: Register the Provider in the Factory

Integrate your new module and struct into the summarizer manager [mod.rs](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/src/summarizer/mod.rs).

1. **Register the Module**:
   Add the public module declaration at the top of the file:
   ```rust
   pub mod your_provider;
   ```

2. **Add Configuration Builder Helper**:
   Define a configuration parser helper that checks and returns configuration tuples:
   ```rust
   fn build_your_provider_config(
       api_key: &str,
       model: &str,
       url: &Option<String>,
   ) -> anyhow::Result<ProviderConfigOutput> {
       if model.is_empty() {
           anyhow::bail!("Model is required: add [your_provider] section with 'model' in asum.toml");
       }
       if api_key.is_empty() {
           anyhow::bail!("API key is required: add 'api_key' to [your_provider] section in asum.toml");
       }
       Ok((
           model.to_string(),
           url.clone(),
           Some(api_key.to_string()),
           None,
           None,
           "your_provider",
       ))
   }
   ```

3. **Update `get_summarizer` Factory**:
   Extract credentials in the configuration matching step:
   ```rust
   ProviderConfig::YourProvider {
       api_key,
       model,
       url,
   } => build_your_provider_config(api_key, model, url)?,
   ```
   Map the provider name to instantiate your provider struct in the factory match:
   ```rust
   "your_provider" => {
       Ok(Box::new(your_provider::YourProviderProvider::new(ai_config)) as Box<dyn Summarizer>)
   }
   ```

---

## Step 4: Update Configuration Files

Document the new configuration options for users.

1. Add the new provider section to [asum.toml](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml) and [asum.toml.example](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/asum.toml.example).
2. Document the parameters (like `model`, `api_key`, and custom endpoints `url`) under comments.
3. List the new provider in the active options under the `[general]` settings comments.

---

## Step 5: Write Tests

As per the strict project convention specified in [AGENTS.md](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/AGENTS.md), unit and integration tests must **never** be placed inside `./src`.

1. **Create Provider Tests**:
   Create a new test file under `tests/` directory (e.g., `tests/your_provider_test.rs`).
   - Write tests checking initialization, input cleaning, error fallback responses, and custom endpoints.
   - Use mock servers or mock responses where appropriate to simulate HTTP request scenarios.

2. **Update Configuration Tests**:
   Update `tests/config_test.rs` to include tests validating invalid/missing properties for the new provider.
