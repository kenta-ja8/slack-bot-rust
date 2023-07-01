use serde_json::json;

use crate::model::config::Config;
use anyhow::Result;

pub struct SlackClient<'a> {
    config: &'a Config,
}

impl<'a> SlackClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn post_message(&self, text: String) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://slack.com/api/chat.postMessage";

        let post_body = json!({
          "channel": self.config.slack_channel,
          "text": text,
          "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": text}}],
          "unfurl_links": false,
        });

        client
            .post(url)
            .bearer_auth(&self.config.slack_bot_token)
            .json(&post_body)
            .send()
            .await?;

        Ok(())
    }
}
