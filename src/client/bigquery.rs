use std::{collections::HashMap, str::FromStr, sync::Arc};

use chrono::{Datelike, Duration, NaiveDateTime};
use google_bigquery2::api::TableRow;

use crate::model::{
    config::Config,
    gcp_cost::{CostReport, ServiceToCostReportMap},
};
use anyhow::{anyhow, Ok, Result};

static DEFAULT_CREDENTIAL_PATH: &str = ".config/gcloud/application_default_credentials.json";

pub struct BigqueryClient {
    config: Arc<Config>,
}

impl<'a> BigqueryClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn get_cost(
        &self,
        start_of_target_date: NaiveDateTime,
    ) -> Result<(ServiceToCostReportMap, f64)> {
        let yesterday_from_target_date = start_of_target_date - Duration::days(1);
        let start_of_month = start_of_target_date
            .with_day(1)
            .ok_or(anyhow!("Failed to get the start_of_month"))?;

        let start_of_next_month = start_of_month
            .with_month(start_of_month.month() + 1)
            .ok_or(anyhow!("Invalid month value"))?;

        let auth = get_auth(self.config.gcp_credential_path.clone()).await?;

        let query = format!(
            "
SELECT service.description, cost, datetime(_PARTITIONTIME)  FROM `{}` 
WHERE TIMESTAMP(\"{}\") <= TIMESTAMP_TRUNC(_PARTITIONTIME, DAY)
AND TIMESTAMP_TRUNC(_PARTITIONTIME, DAY) < TIMESTAMP(\"{}\")
AND project.id = \"{}\"",
            self.config.gcp_bigquery_cost_table,
            start_of_month.format("%Y-%m-%d").to_string(),
            start_of_next_month.format("%Y-%m-%d").to_string(),
            self.config.gcp_project_id,
        );
        // println!("{}", query);

        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        let hub = google_bigquery2::Bigquery::new(client, auth);
        let mut req = google_bigquery2::api::QueryRequest::default();
        req.query = Some(query.to_string());
        req.use_legacy_sql = Some(false);

        let result = hub
            .jobs()
            .query(req, &self.config.gcp_project_id)
            .doit()
            .await?;
        let query_res: google_bigquery2::api::QueryResponse = result.1;

        let mut date_to_service_cost: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut month_total = 0.0;
        for row in query_res.rows.ok_or(anyhow!("Failed to get row"))? {
            let service: String = extract_value(&row, 0)?;
            // println!("{}", service);
            let cost: f64 = extract_value(&row, 1)?;
            // println!("{}", cost);
            let date: String = extract_value(&row, 2)?;
            // println!("{}", date);

            let service_map = date_to_service_cost
                .entry(date)
                .or_insert_with(HashMap::new);
            let current_cost = service_map.entry(service).or_insert(0.0);
            *current_cost += cost;
            month_total += cost;
        }
        // println!("{:?}", date_to_service_cost);

        let mut service_to_cost_report: ServiceToCostReportMap = ServiceToCostReportMap::new();
        for (service, cost) in date_to_service_cost
            .get(&(start_of_target_date.format("%Y-%m-%d").to_string() + "T00:00:00"))
            .ok_or(anyhow!("Failed to get service_to_cost"))?
        {
            let cost_2day_ago = date_to_service_cost
                .get(&(yesterday_from_target_date.format("%Y-%m-%d").to_string() + "T00:00:00"))
                .and_then(|server_to_cost| server_to_cost.get(service));

            let diff_cost = cost - cost_2day_ago.unwrap_or(&0.0);
            let diff_rate = match cost_2day_ago {
                Some(&cost_2day_ago) if cost_2day_ago != 0.0 => Some(cost / cost_2day_ago),
                _ => None,
            };

            service_to_cost_report.insert(
                service.to_string(),
                CostReport {
                    diff_rate,
                    diff_cost,
                    cost: cost.to_owned(),
                },
            );
        }
        // println!("{:?}", service_to_cost_report);

        Ok((service_to_cost_report, month_total))
    }
}

fn extract_value<T: FromStr>(row: &TableRow, index: usize) -> Result<T> {
    let value_str = row
        .f
        .as_ref()
        .and_then(|f| f.get(index))
        .and_then(|field| field.v.as_ref())
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow!("Missing value at index {}", index))?;

    value_str
        .parse::<T>()
        .map_err(|_| anyhow!("Failed to parse value: {}", value_str))
}

async fn get_auth(
    gcp_credential_path: Option<String>,
) -> Result<
    google_bigquery2::oauth2::authenticator::Authenticator<
        google_bigquery2::hyper_rustls::HttpsConnector<hyper::client::HttpConnector>,
    >,
> {
    let file_path = match gcp_credential_path {
        Some(path) => path.clone(),
        None => dirs::home_dir()
            .map(|path| path.join(DEFAULT_CREDENTIAL_PATH))
            .and_then(|path| path.to_str().map(|s| s.to_string()))
            .ok_or(anyhow!("Failed to get the default path"))?,
    };

    if std::path::Path::new(&file_path).exists() {
        let secret = google_bigquery2::oauth2::read_authorized_user_secret(file_path).await?;
        let auth = google_bigquery2::oauth2::AuthorizedUserAuthenticator::builder(secret)
            .build()
            .await?;
        return Ok(auth);
    }
    let opts = google_bigquery2::oauth2::ApplicationDefaultCredentialsFlowOpts::default();
    let auth =
        match google_bigquery2::oauth2::ApplicationDefaultCredentialsAuthenticator::builder(opts)
            .await
        {
            google_bigquery2::oauth2::authenticator::ApplicationDefaultCredentialsTypes::ServiceAccount(auth) => auth
                .build()
                .await
                .expect("Unable to create service account authenticator"),
            google_bigquery2::oauth2::authenticator::ApplicationDefaultCredentialsTypes::InstanceMetadata(auth) => {
                auth.build()
                    .await
                    .expect("Unable to create instance metadata authenticator")
            }
        };
    Ok(auth)
}
