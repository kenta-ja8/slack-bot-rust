use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub slack_bot_token: String,
    pub slack_channel: String,
    pub platform: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let _ = dotenv();
    let config = Config {
        slack_bot_token: env::var("SLACK_BOT_TOKEN")?,
        slack_channel: env::var("SLACK_CHANNEL")?,
        platform: env::var("CLOUD_RUN_EXECUTION").unwrap_or("UNKNOWN".to_string()),
    };
    Ok(config)
}
