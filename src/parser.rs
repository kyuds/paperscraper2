use std::{
    fmt,
    option::Option
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use reqwest;
use quick_xml::de::from_str;
use serde::{
    de::{Visitor, MapAccess}, 
    Deserialize, 
    Deserializer
};

use crate::config::Config;

macro_rules! arxiv_url {
    () => { concat!(
        "https://export.arxiv.org/api/query/?search_query=",
        "%28{}%29+AND+submittedDate:[{}+TO+{}]&start={}&max_results={}"
    ) }
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

    fn create_query_url(&self, date: Option<DateTime<Utc>>, start: i32) -> String {
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
        format!(arxiv_url!(), categories, d0, d1, start, self.config.num_entries)
    }

    fn get_raw_xml(&self, date: Option<DateTime<Utc>>, start: i32) -> String {
        let url = self.create_query_url(date, start);
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

    pub fn get_arxiv_results(&self, date: Option<DateTime<Utc>>) -> Vec<ArxivResult> {
        let mut results: Vec<ArxivResult> = Vec::new();
        for page in 0..self.config.num_pages {
            let start = self.config.num_entries * page;
            let xml = self.get_raw_xml(date, start);
            let parsed: ArxivDocument = match from_str(xml.as_str()) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to parse xml data: {}", e);
                    println!("{}", xml);
                    ArxivDocument::default()
                }
            };
            let mut page_results = parsed.entries.into_iter()
                .map(ArxivResult::from_entry)
                .collect::<Vec<_>>();
            if page_results.is_empty() {
                break;
            }
            println!("epoch {}, documents {}", page, page_results.len());
            results.append(&mut page_results);
        }
        results
    }
}

// Arxiv Data Model

#[derive(Debug)]
pub struct ArxivResult {
    pub title: String,
    pub summary: String,
    pub authors: Vec<String>,
    pub published: DateTime<Utc>,
    pub link: String
}

impl ArxivResult {
    fn new(title: String, summary: String, authors: Vec<String>, published: DateTime<Utc>, link: String) -> Self {
        ArxivResult {
            title,
            summary,
            authors,
            published,
            link
        }
    }

    fn from_entry(entry: ArxivEntry) -> Self {
        let published: DateTime<Utc> = DateTime::parse_from_rfc3339(&entry.published)
            .map(|dt| dt.with_timezone(&Utc)) 
            .unwrap_or_else(|_err| {
                eprintln!("Failed to parse published date: {}", _err);
                Utc.timestamp_opt(0, 0).unwrap()
            });

        Self::new(
            entry.title.replace("\n", " "), 
            entry.summary.replace("\n", " "), 
            entry.authors.into_iter().map(|a| a.name.value).collect::<Vec<_>>(), 
            published, 
            entry.links.into_iter()
                .find(|field| match field.link_type {
                    Some(LinkType::Home) => true,
                    _ => false,
                })
                .map(|field| field.link)
                .unwrap_or_else(|| String::new())
        )
    }
}

// end Arxiv Data Model

// Arxiv Raw XML Model

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct ArxivDocument {
    #[serde(rename = "entry")]
    entries: Vec<ArxivEntry>
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct ArxivEntry {
    title: String,
    summary: String,
    #[serde(rename = "author", flatten, deserialize_with = "de_author")]
    authors: Vec<AuthorField>,
    published: String,
    #[serde(rename = "link", flatten, deserialize_with = "de_link")]
    links: Vec<LinkField>
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct AuthorField {
    name: NameField
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct NameField {
    #[serde(rename = "$text")]
    value: String
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct LinkField {
    #[serde(rename = "@href")]
    link: String,
    #[serde(rename = "@type")]
    link_type: Option<LinkType>
}

#[derive(Debug, Default, PartialEq, Deserialize)]
enum LinkType {
    #[serde(rename = "text/html")]
    Home,
    #[serde(rename = "application/pdf")]
    Pdf,
    #[default]
    Unknown,
}

fn de_author<'de, D>(deserializer: D) -> Result<Vec<AuthorField>, D::Error>
where
    D: Deserializer<'de>,
{
    struct AuthorVisitor;
    impl<'de> Visitor<'de> for AuthorVisitor {
        type Value = Vec<AuthorField>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Map of children elements - filtering for field: `author`")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut links = Vec::<AuthorField>::new();
            while let Some(key) = access.next_key::<String>()? {
                if key == "author" {
                    let var = access.next_value::<AuthorField>().unwrap();
                    links.push(var);
                }
            };
            Ok(links)
        }
    }
    deserializer.deserialize_any(AuthorVisitor{})
}

fn de_link<'de, D>(deserializer: D) -> Result<Vec<LinkField>, D::Error> 
where 
    D: Deserializer<'de>,
{
    struct LinkVisitor;
    impl<'de> Visitor<'de> for LinkVisitor {
        type Value = Vec<LinkField>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Map of children elements - filtering for field: `link`")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut links = Vec::<LinkField>::new();
            while let Some(key) = access.next_key::<String>()? {
                if key == "link" {
                    let var = access.next_value::<LinkField>().unwrap();
                    links.push(var);
                }
            };
            Ok(links)
        }
    }
    deserializer.deserialize_any(LinkVisitor{})
}

// end Arxiv Raw XML Model

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    const ACTUAL: &str = concat!(
        "https://export.arxiv.org/api/query/",
        "?search_query=%28cat:cs.CL+OR+cat:cs.AI+OR+cat:cs.LG+OR+cat:cs.MA%29+AND+",
        "submittedDate:[202412300000+TO+202412310000]&start=0&max_results=500"
    );

    #[test]
    fn test_url_generation() {
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 1).unwrap();
        let parser = ArxivParser::new(Config::default());
        let url = parser.create_query_url(Some(date), 0);
        assert_eq!(url, ACTUAL, "URL improperly formatted");
    }
}
