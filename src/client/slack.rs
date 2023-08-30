use std::sync::Arc;

use serde_json::json;

use crate::model::{
    config::Config,
    openai::{Engine, PaperSummaryModel},
    paper::PaperModel,
};
use anyhow::Result;

pub struct SlackClient {
    config: Arc<Config>,
}

impl<'a> SlackClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn post_message(
        &self,
        paper: &PaperModel,
        answer: &PaperSummaryModel,
        engine: &Engine,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://slack.com/api/chat.postMessage";

        let mut text = format!("*{}*\n", answer.title);
        for s in &answer.summary {
            text.push_str(&format!(" • {}\n", s));
        }

        let info = format!(
            "Powered by {}  / Running on {}\n",
            engine.to_string(),
            &self.config.platform
        );

        let post_body = json!({
          "channel": self.config.slack_channel,
          "attachments": [
            {
              "mrkdwn_in": ["text"],
              "color": "#3560a6",
              "title": paper.title,
              "title_link": paper.url,
              "text": text,
              "footer": info,
            }
         ]
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
