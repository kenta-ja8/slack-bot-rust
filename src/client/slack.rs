use std::sync::Arc;

use chrono::NaiveDateTime;
use serde_json::json;

use crate::model::{
    config::Config,
    gcp_cost::ServiceToCostReportMap,
    openai::{Engine, PaperSummaryModel},
    paper::PaperModel,
};
use anyhow::Result;

static SLACK_POST_URL: &str = "https://slack.com/api/chat.postMessage";

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
            .post(SLACK_POST_URL)
            .bearer_auth(&self.config.slack_bot_token)
            .json(&post_body)
            .send()
            .await?;

        Ok(())
    }
    pub async fn post_daily_cost(
        &self,
        service_to_cost: ServiceToCostReportMap,
        monthly_total: f64,
        target_date: NaiveDateTime,
    ) -> Result<()> {
        let client = reqwest::Client::new();

        let mut fields: Vec<_> = service_to_cost
            .iter()
            .map(|(k, v)| {
                let cost = format!("${:.0}", v.cost);
                let percent = v
                    .diff_rate
                    .and_then(|r| {
                        let percent = r * 100.0 - 100.0;
                        if percent > 0.0 {
                            Some(format!("+{:.0}%", percent))
                        } else {
                            Some(format!("{:.0}%", percent))
                        }
                    })
                    .unwrap_or("-".to_string());
                json!( {
                    "title": k,
                    "value": format!("{} ({})", cost, percent),
                    "short": true
                })
            })
            .collect();

        fields.sort_by(|a, b| a["title"].as_str().cmp(&b["title"].as_str()));

        let title = format!("*Cost Report*");
        let monthly_total_str = format!(
            "{}:  *${:.0}*",
            target_date.format("%Y/%m"),
            monthly_total,
        );
        let daily_total_str = format!(
            "{}:  ${:.0}",
            target_date.format("%Y/%m/%d"),
            service_to_cost.values().map(|v| v.cost).sum::<f64>()
        );

        let project = format!("Project:  {}", &self.config.gcp_project_id);
        let remark = format!("\n_※ Cost from 09:00 JST to 09:00 JST the following day (compared to the previous day)._");
        let pretext = format!(
            "{}\n{}\n{}\n{}",
            title, project, monthly_total_str, daily_total_str
        );
        let footer = format!(
            "Data sourced from {} / Running on {}\n",
            &self.config.gcp_bigquery_cost_table, &self.config.platform
        );
        fields.push(json!({
              "value": remark,
              "short": false
        }));
        let post_body = json!({
          "channel": self.config.slack_channel,
          "attachments": [
            {
              "mrkdwn_in": ["text"],
              "pretext": pretext,
              "color": "#cdcdcd",
              "fields": fields,
              "footer": footer,
            }
         ]
        });
        // println!("{}", post_body);

        client
            .post(SLACK_POST_URL)
            .bearer_auth(&self.config.slack_bot_token)
            .json(&post_body)
            .send()
            .await?;

        Ok(())
    }
}
