use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::fs;
use std::str::FromStr;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let mut db_path = home::home_dir().expect("Could not find home directory");
    db_path.push(".asum");

    // Ensure directory exists
    if !db_path.exists() {
        fs::create_dir_all(&db_path).expect("Could not create database directory");
    }

    db_path.push("asum.db");
    let db_url = format!("sqlite:{}", db_path.to_str().expect("Invalid path"));

    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Create config table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Seed default values if not present
    seed_defaults(&pool).await?;

    Ok(pool)
}

async fn seed_defaults(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let defaults = [
        ("active_provider", "ollama"),
        ("ollama_url", "http://localhost:11434/api/generate"),
        ("ollama_model", "llama3.2:1b"),
        ("gemini_api_key", ""),
        ("gemini_api_model", "gemini-2.0-flash"),
        ("ai_temperature", "0.1"),
        ("ai_num_predict", "250"),
        ("ai_top_p", "0.9"),
        ("max_diff_length", "1000000"),
    ];

    for (key, value) in defaults {
        sqlx::query("INSERT OR IGNORE INTO config (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn get_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM config WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.0))
}
