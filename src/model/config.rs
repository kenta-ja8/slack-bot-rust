use anyhow::{Result};
use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub arxiv_query: String,
    pub openai_api_key: String,
    pub slack_bot_token: String,
    pub slack_channel: String,
    pub platform: String,
}

pub fn load_config() -> Result<Config> {
    let _ = dotenv();
    let config = Config {
        arxiv_query: env::var("ARXIV_QUERY")
            .unwrap_or("llm OR \"generative ai\" OR \"visual recognition\"".to_string()),
        openai_api_key: env::var("OPENAI_API_KEY")?,
        slack_bot_token: env::var("SLACK_BOT_TOKEN")?,
        slack_channel: env::var("SLACK_CHANNEL")?,
        platform: env::var("CLOUD_RUN_EXECUTION").unwrap_or("UNKNOWN".to_string()),
    };
    Ok(config)
}
