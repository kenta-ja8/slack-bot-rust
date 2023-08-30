mod client;
mod model;

use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use tokio::task;

async fn execute() -> Result<()> {
    let config = Arc::new(model::config::load_config()?);
    let openai = Arc::new(client::openai::OpenAiClient::new(Arc::clone(&config)));
    let slack = Arc::new(client::slack::SlackClient::new(Arc::clone(&config)));
    let arxiv = Arc::new(client::arxiv::ArxivClient::new(Arc::clone(&config)));

    let papers = arxiv.search_past_5_to_6_days().await?;
    if papers.is_empty() {
        println!("not found paper");
        return Ok(());
    }

    let handles = papers
        .into_iter()
        .map(|p| {
            let openai = Arc::clone(&openai);
            let slack = Arc::clone(&slack);
            task::spawn(async move {
                // println!("start spawn");

                let engine = model::openai::Engine::Gpt4;
                let paper_summary = openai.summarize_paper(&p, &engine).await?;
                slack.post_message(&p, &paper_summary, &engine).await?;
                // println!("end spawn");
                Ok::<(), anyhow::Error>(())
            })
        })
        .collect::<Vec<_>>();

    join_all(handles)
        .await
        .into_iter()
        .map(|res| res.map_err(anyhow::Error::from).and_then(|x| x))
        .collect::<Result<Vec<_>>>()?;

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
