use dotenvy;
use std::{env, process};
use chrono::Utc;

const RAW_FOLDER_PREFIX: &str = "raw";
const PROCESSED_FOLDER_PREFIX: &str = "processed";

#[derive(Debug)]
pub struct ArxivConfig {
    pub num_entries: i32,
    pub num_pages: i32,
    pub date_offset: i32,
    pub categories: Vec<String>,
}

#[allow(dead_code)]
impl ArxivConfig {
    pub fn default() -> Self {
        ArxivConfig {
            num_entries: 50,
            num_pages: 10,
            date_offset: 1,
            categories: vec![
                String::from("cs.CL"),
                String::from("cs.AI"),
                String::from("cs.LG"),
                String::from("cs.MA")
            ]
        }
    }

    pub fn new(num_entries: i32, num_pages: i32, date_offset: i32, categories: Vec<String>) -> Self {
        ArxivConfig {
            num_entries,
            num_pages,
            date_offset,
            categories
        }
    }

    pub fn from_env() -> Self {
        dotenvy::from_filename("paperscraper.env").unwrap();
        let num_entries = get_positive_i32_from_env("NUM_ENTRIES");
        let num_pages = get_positive_i32_from_env("NUM_PAGES");
        let date_offset = get_positive_i32_from_env("DATE_OFFSET");
        let categories: Vec<String> = env::var("CATEGORIES")
            .expect("CATEGORIES not found in env")
            .split_whitespace()
            .map(String::from)
            .collect();
        Self::new(num_entries, num_pages, date_offset, categories)
    }
}

fn get_positive_i32_from_env(key: &str) -> i32 {
    let var: i32 = env::var(key)
        .expect(format!("{} not found in env", key).as_str())
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("Failed to parse NUM_ENTRIES as i32");
            process::exit(1);
        });
    assert!(var > 0, "{} must be positive", key);
    var
}

#[derive(Debug)]
pub struct NameConfig {
    pub bucket: String,
    key: String
}

impl NameConfig {
    pub fn new(bucket: &str, key: &str) -> Self {
        NameConfig {
            bucket: bucket.to_string(),
            key: key.to_string()
        }
    }

    pub fn default(bucket: &str) -> Self {
        let key = Utc::now().format("%y%m%d%H%M%S").to_string();
        Self::new(bucket, &key)
    }

    pub fn raw_jsonl_path(&self) -> String {
        format!("{}/raw_{}.jsonl", RAW_FOLDER_PREFIX, self.key)
    }

    pub fn processed_jsonl_path(&self) -> String {
        format!("{}/processed_{}.jsonl", PROCESSED_FOLDER_PREFIX, self.key)
    }
}
