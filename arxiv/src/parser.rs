use std::option::Option;
use chrono::{Duration, Utc, DateTime};
use reqwest;

use crate::config::Config;

// URL query creator
macro_rules! arxiv_url {
    () => { "https://export.arxiv.org/api/query/?search_query=%28{}%29+AND+submittedDate:[{}+TO+{}]&max_results={}" }
}

#[derive(Debug)]
pub struct ArxivParser {
    config: Config
}

impl ArxivParser {
    pub fn new(config: Config) -> Self {
        ArxivParser {
            config
        }
    }

    fn create_query_url(&self, date: Option<DateTime<Utc>>) -> String {
        // search categories.
        let categories = self.config.categories.iter()
            .map(|cat| format!("cat:{}", cat))
            .collect::<Vec<_>>()
            .join("+OR+");

        // search dates. 
        let offset = self.config.date_offset as i64;
        let t = date.unwrap_or_else(Utc::now);
        let d0 = format!("{}0000", (t - Duration::days(offset + 1)).format("%Y%m%d"));
        let d1 = format!("{}0000", (t - Duration::days(offset)).format("%Y%m%d"));

        // format using a named macro
        format!(arxiv_url!(), categories, d0, d1, self.config.num_entries)
    }

    fn get_raw_xml(&self) -> String {
        let url = self.create_query_url(None);
        let response = match reqwest::blocking::get(url) {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Failed to fetch data: {}", e);
                return String::new();
            }
        };
        match response.text() {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Failed to read response body: {}", e);
                return String::new();
            }
        }
    }

    pub fn get_arxiv_results(&self) -> String {
        self.get_raw_xml()
    }
}

#[derive(Debug)]
pub struct ArxivResult {
    pub title: String,
    pub content: String,
    pub link: String,
    pub authors: Vec<String>,
    pub published: DateTime<Utc>
}

impl ArxivResult {
    fn new(title: String, content: String, link: String, authors: Vec<String>, published: DateTime<Utc>) -> Self {
        ArxivResult {
            title,
            content,
            link,
            authors,
            published
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    static ACTUAL: &str = "https://export.arxiv.org/api/query/?search_query=%28cat:cs.CL+OR+cat:cs.AI+OR+cat:cs.LG+OR+cat:cs.MA%29+AND+submittedDate:[202412300000+TO+202412310000]&max_results=500";

    #[test]
    fn test_url_generation() {
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 1).unwrap();
        let parser = ArxivParser::new(Config::default());
        let url = parser.create_query_url(Some(date));
        assert_eq!(url, ACTUAL, "URL improperly formatted");
    }
}
