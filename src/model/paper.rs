#[derive(Debug)]
pub struct PaperModel{
    pub url: String,
    pub published: String,
    pub title: String,
    pub summary: String,
    pub authors: Vec<String>,
}
