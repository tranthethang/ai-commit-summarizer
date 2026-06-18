use crate::summarizer::AIConfig;
use std::time::Duration;

/// Creates a new HTTP client with a 120-second timeout.
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("Failed to build HTTP client")
}

/// Injects the git diff into the provided prompt template.
/// Replaces the `{{diff}}` placeholder with the actual diff content.
pub fn generate_prompt(prompt_template: &str, diff: &str) -> String {
    prompt_template.replace("{{diff}}", diff)
}

/// Cleans the raw AI response by removing empty lines and
/// lines that echo input diff instructions.
pub fn clean_ai_response(raw: &str) -> anyhow::Result<String> {
    let cleaned = raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.to_lowercase().contains("diff to analyze")
                && !l.to_lowercase().contains("input diff")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if cleaned.is_empty() {
        anyhow::bail!("AI generated an empty or invalid message.");
    }

    Ok(cleaned)
}

/// Logs the system and user prompts in verbose mode.
pub fn log_verbose_prompt(system_prompt: &str, user_prompt: &str) {
    eprintln!("================ PROMPT ================");
    eprintln!("*** System Prompt ***\n{}", system_prompt);
    eprintln!("*** User Prompt ***\n{}", user_prompt);
    eprintln!("========================================");
}

/// Logs the raw API response JSON in verbose mode.
pub fn log_verbose_response(body: &str) {
    eprintln!("================ RESPONSE JSON ================");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
        if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
            eprintln!("{}", pretty);
        } else {
            eprintln!("{}", body);
        }
    } else {
        eprintln!("{}", body);
    }
    eprintln!("===============================================");
}

/// Helper to construct the Gemini-style JSON payload.
pub fn build_gemini_payload(
    system_prompt: &str,
    user_prompt: &str,
    config: &AIConfig,
) -> serde_json::Value {
    serde_json::json!({
        "system_instruction": {
            "parts": [{
                "text": system_prompt
            }]
        },
        "contents": [{
            "role": "user",
            "parts": [{
                "text": user_prompt
            }]
        }],
        "generationConfig": {
            "temperature": config.temperature,
            "topP": config.top_p,
            "maxOutputTokens": config.num_predict,
        }
    })
}

/// Helper to check if the response status is successful, returning a detailed error if not.
pub async fn check_response_status(
    response: reqwest::Response,
    provider_name: &str,
) -> anyhow::Result<reqwest::Response> {
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!(
            "{} returned error: {} - {}",
            provider_name,
            status,
            error_text
        );
    }
    Ok(response)
}

/// Helper to extract and clean the commit message from a Gemini-style JSON response.
pub fn parse_gemini_response(res_text: &str, verbose: bool) -> anyhow::Result<String> {
    if verbose {
        log_verbose_response(res_text);
    }
    let res_json: serde_json::Value = serde_json::from_str(res_text)?;
    let commit_msg = res_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim();
    clean_ai_response(commit_msg)
}

/// Helper to extract and clean the commit message from an OpenAI-style JSON response.
pub fn parse_openai_response(res_text: &str, verbose: bool) -> anyhow::Result<String> {
    if verbose {
        log_verbose_response(res_text);
    }
    let res_json: serde_json::Value = serde_json::from_str(res_text)?;
    let commit_msg = res_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim();
    clean_ai_response(commit_msg)
}
