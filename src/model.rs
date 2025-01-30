use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// used for both raw arXiv parsing and final gen-AI invoked paper abstract summaries.

#[derive(Debug, Deserialize, Serialize)]
pub struct ArxivResult {
    pub id: usize,
    pub title: String,
    pub summary: String,
    pub authors: Vec<String>,
    pub published: DateTime<Utc>,
    pub link: String
}

impl ArxivResult {
    pub fn new(
        id: usize, 
        title: String, 
        summary: String, 
        authors: Vec<String>, 
        published: DateTime<Utc>, 
        link: String
    ) -> Self {
        ArxivResult {
            id,
            title,
            summary,
            authors,
            published,
            link
        }
    }
}
