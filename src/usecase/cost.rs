use crate::client::{bigquery::BigqueryClient, slack::SlackClient};

use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDateTime, Utc};

pub struct CostUsecase {
    slack_client: Arc<SlackClient>,
    bigquery_client: Arc<BigqueryClient>,
}

impl<'a> CostUsecase {
    pub fn new(slack_client: Arc<SlackClient>, bigquery_client: Arc<BigqueryClient>) -> Self {
        Self {
            slack_client,
            bigquery_client,
        }
    }

    pub async fn notify_daily_cost(&self) -> Result<()> {
        // BigQueryへのコストエクスポートは下記の特徴があるため、呼び出しは日本時間で１５時以降が良い
        // ・UTC時間の日付毎にパーティションが切られている -> 日本時間９時が切り替わり時刻
        // ・コストは１時間毎に集計されている
        // ・BigQueryへのコストレコードの追加は３〜５時間程のラグがある -> １２時〜１４時の間に追加される

        let yesterday = Utc::now().naive_utc() - Duration::days(1);
        let target_date = NaiveDateTime::new(
            yesterday.into(),
            chrono::NaiveTime::from_hms_opt(0, 0, 0).ok_or(anyhow!("Failed to get target_date"))?,
        );

        let (service_to_cost_report, month_total) =
            self.bigquery_client.get_cost(target_date).await?;

        self.slack_client
            .post_daily_cost(service_to_cost_report, month_total, target_date)
            .await?;

        Ok(())
    }
}
