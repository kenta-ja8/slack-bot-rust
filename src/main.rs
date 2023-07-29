mod client;
mod model;

use anyhow::Result;

async fn execute() -> Result<()> {
    let config = model::config::load_config()?;
    let arxiv = client::arxiv::ArxivClient::new(&config);
    let openai = client::openai::OpenAiClient::new(&config);
    let slack = client::slack::SlackClient::new(&config);

    let papers = arxiv.search_past_5_to_6_days().await?;
    if papers.len() == 0 {
        println!("not found paper");
        return Ok(());
    }

    let engine = model::openai::Engine::Gpt4;
    for p in papers {
        let paper_summary = openai.summarize_paper(&p, &engine).await?;
        slack.post_message(&p, &paper_summary, &engine).await?;
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
