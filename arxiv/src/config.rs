use dotenvy;
use std::{env, process};

#[derive(Debug)]
pub struct Config {
    pub num_entries: i32,
    pub date_offset: i32,
    pub categories: Vec<String>,
}

#[allow(dead_code)]
impl Config {
    pub fn default() -> Self {
        Config {
            num_entries: 500,
            date_offset: 1,
            categories: vec![
                String::from("cs.CL"),
                String::from("cs.AI"),
                String::from("cs.LG"),
                String::from("cs.MA")
            ]
        }
    }

    pub fn new(num_entries: i32, date_offset: i32, categories: Vec<String>) -> Self {
        Config {
            num_entries,
            date_offset,
            categories
        }
    }

    pub fn from_env() -> Self {
        dotenvy::from_filename("paperscraper.env").unwrap();
        let num_entries = get_positive_i32_from_env("NUM_ENTRIES");
        let date_offset = get_positive_i32_from_env("DATE_OFFSET");
        let categories: Vec<String> = env::var("CATEGORIES")
            .expect("CATEGORIES not found in env")
            .split_whitespace()
            .map(String::from)
            .collect();
        Self::new(num_entries, date_offset, categories)
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
