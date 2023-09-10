use anyhow::Result;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub cmd: String,
    pub arxiv_query: String,
    pub openai_api_key: String,
    pub slack_bot_token: String,
    pub slack_channel: String,
    pub platform: String,
    pub gcp_credential_path: Option<String>,
    pub gcp_project_id: String,
    pub gcp_bigquery_cost_table: String,
}

pub fn load_config() -> Result<Config> {
    let _ = dotenv();
    let config = Config {
        cmd: env::var("CMD")?,
        arxiv_query: env::var("ARXIV_QUERY")
            .unwrap_or("llm OR \"generative ai\" OR \"visual recognition\"".to_string()),
        openai_api_key: env::var("OPENAI_API_KEY")?,
        slack_bot_token: env::var("SLACK_BOT_TOKEN")?,
        slack_channel: env::var("SLACK_CHANNEL")?,
        platform: env::var("CLOUD_RUN_EXECUTION").unwrap_or("UNKNOWN".to_string()),
        gcp_credential_path: env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
        gcp_project_id: env::var("GCP_PROJECT_ID")?,
        gcp_bigquery_cost_table: env::var("GCP_BIGQUERY_COST_TABLE")?,
    };
    Ok(config)
}
