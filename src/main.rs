mod client;
mod model;
mod usecase;

use std::sync::Arc;

use anyhow::Result;

async fn execute() -> Result<()> {
    let config = Arc::new(model::config::load_config()?);
    let openai_client = Arc::new(client::openai::OpenAiClient::new(Arc::clone(&config)));
    let slack_client = Arc::new(client::slack::SlackClient::new(Arc::clone(&config)));
    let bigquery_client = Arc::new(client::bigquery::BigqueryClient::new(Arc::clone(&config)));
    let arxiv_client = Arc::new(client::arxiv::ArxivClient::new(Arc::clone(&config)));

    let paper_usecase = Arc::new(usecase::paper::PaperUsecase::new(
        Arc::clone(&slack_client),
        Arc::clone(&arxiv_client),
        Arc::clone(&openai_client),
    ));
    let cost_notification_usecase = Arc::new(usecase::cost::CostUsecase::new(
        Arc::clone(&slack_client),
        Arc::clone(&bigquery_client),
    ));

    match config.cmd.as_str() {
        "notify_paper" => paper_usecase.notify_paper().await?,
        "notify_daily_cost" => cost_notification_usecase.notify_daily_cost().await?,
        cmd => Err(anyhow::anyhow!("Unknown command: {}", cmd))?,
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Start Job");

    match execute().await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            e.backtrace();
            std::process::exit(1);
        }
    };

    println!("End Job");
}
