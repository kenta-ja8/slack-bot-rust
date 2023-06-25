use crate::model::config::Config;
use crate::model::paper::PaperModel;
use arxiv::ArxivQueryBuilder;
use chrono::{DateTime, Duration, Utc};

pub struct ArxivClient<'a> {
    config: &'a Config,
}

impl<'a> ArxivClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn search_yesterday_paper(
        &self,
    ) -> Result<Vec<PaperModel>, Box<dyn std::error::Error>> {
        let duration_in_days = 1;
        let max_paper = 2;

        let query = ArxivQueryBuilder::new()
            .search_query(&self.config.arxiv_query)
            .start(0)
            .max_results(30)
            .sort_by("submittedDate")
            .build();
        let arxivs = arxiv::fetch_arxivs(query).await?;

        let now = Utc::now().naive_utc();
        let start_of_today = now
            .date()
            .and_hms_opt(0, 0, 0)
            .ok_or("Failed to create date.")?;
        let start_of_yesterday = start_of_today - Duration::days(duration_in_days);

        let mut papers = vec![];
        for arxiv in arxivs {
            if papers.len() >= max_paper {
                break;
            }
            let published = DateTime::parse_from_rfc3339(&arxiv.published)?;
            if !(start_of_yesterday <= published.naive_utc()
                && published.naive_utc() < start_of_today)
            {
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
