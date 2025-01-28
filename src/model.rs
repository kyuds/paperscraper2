use serde::Serialize;
use chrono::{DateTime, Utc};

// used for both raw arXiv parsing and final gen-AI invoked paper abstract summaries.

#[derive(Debug, Serialize)]
pub struct ArxivResult {
    pub id: usize,
    pub title: String,
    pub summary: String,
    pub authors: Vec<String>,
    pub published: DateTime<Utc>,
    pub link: String
}
