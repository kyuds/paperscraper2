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
    pub fn record_id(&self) -> String {
        format!("ARXIV{:06}", self.id)
    }

    // pub fn id_from_record_id(rid: &str) -> Option<usize> {
    //     if let Some(stripped) = rid.strip_prefix("ARXIV") {
    //         match stripped.parse::<usize>() {
    //             Ok(n) => Some(n),
    //             Err(e) => {
    //                 eprintln!("Failed to convert record id ({}): {}", rid, e);
    //                 None
    //             },
    //         }
    //     } else {
    //         eprintln!("No prefix (ARXIV) found: {}", rid);
    //         None
    //     }
    // }
}
