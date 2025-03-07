use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessedResult {
    pub id: usize,
    pub title: String,
    pub original: String,
    pub summary: String,
    pub authors: Vec<String>,
    pub published: DateTime<Utc>,
    pub link: String
}

impl ProcessedResult {
    pub fn new(
        id: usize, 
        title: String, 
        original: String,
        summary: String, 
        authors: Vec<String>, 
        published: DateTime<Utc>, 
        link: String
    ) -> Self {
        ProcessedResult {
            id,
            title,
            original,
            summary,
            authors,
            published,
            link
        }
    }

    pub fn from_result(
        original: ArxivResult,
        summary: String
    ) -> Self {
        ProcessedResult {
            id: original.id,
            title: original.title,
            original: original.summary,
            summary,
            authors: original.authors,
            published: original.published,
            link: original.link
        }
    }
}
