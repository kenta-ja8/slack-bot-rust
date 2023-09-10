use crate::{
    client::{arxiv::ArxivClient, openai::OpenAiClient, slack::SlackClient},
    model,
};

use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use tokio::task;

pub struct PaperUsecase {
    slack_client: Arc<SlackClient>,
    arxiv_client: Arc<ArxivClient>,
    openai_client: Arc<OpenAiClient>,
}

impl<'a> PaperUsecase {
    pub fn new(
        slack_client: Arc<SlackClient>,
        arxiv_client: Arc<ArxivClient>,
        openai_client: Arc<OpenAiClient>,
    ) -> Self {
        Self {
            slack_client,
            arxiv_client,
            openai_client,
        }
    }

    pub async fn notify_paper(&self) -> Result<()> {
        let papers = self.arxiv_client.search_past_5_to_6_days().await?;
        if papers.is_empty() {
            println!("not found paper");
            return Ok(());
        }

        let handles = papers
            .into_iter()
            .map(|p| {
                let openai = Arc::clone(&self.openai_client);
                let slack = Arc::clone(&self.slack_client);
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
}
