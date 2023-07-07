use core::fmt;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Engine {
    Gpt3_5Turbo,
    Gpt4,
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Engine::Gpt3_5Turbo => write!(f, "gpt-3.5-turbo"),
            Engine::Gpt4 => write!(f, "gpt-4"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaperSummaryModel {
    pub title: String,
    pub summary: Vec<String>,
}
