use crate::model::config::Config;
use crate::model::paper::PaperModel;
use anyhow::Result;
use arxiv::ArxivQueryBuilder;
use chrono::{DateTime, Duration, Utc};

pub struct ArxivClient<'a> {
    config: &'a Config,
}

impl<'a> ArxivClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn search_past_5_to_6_days(&self) -> Result<Vec<PaperModel>> {
        let max_paper = 2;

        let query = ArxivQueryBuilder::new()
            .search_query(&self.config.arxiv_query)
            .start(0)
            .max_results(100)
            .sort_by("submittedDate")
            .build();
        let arxivs = arxiv::fetch_arxivs(query).await?;

        let now = Utc::now().naive_utc();
        let date_to = now - Duration::days(5);
        let date_from = date_to - Duration::days(1);

        let mut papers = vec![];
        for arxiv in arxivs {
            if papers.len() >= max_paper {
                break;
            }
            let published = DateTime::parse_from_rfc3339(&arxiv.published)?;
            if published.naive_utc() < date_from || date_to <= published.naive_utc() {
                continue;
            }
            papers.push(PaperModel {
                url: arxiv.id,
                published: arxiv.published,
                title: arxiv.title.replace("\n", " "),
                summary: arxiv.summary.replace("\n", " "),
                authors: arxiv.authors,
            })
        }

        Ok(papers)
    }
}
