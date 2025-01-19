use dotenvy;
use std::{env, process};

#[derive(Debug)]
pub struct Config {
    pub num_entries: i32,
    pub categories: Vec<String>,
}

#[allow(dead_code)]
impl Config {
    pub fn default() -> Self {
        Config {
            num_entries: 500,
            categories: vec![
                String::from("cs.CL"),
                String::from("cs.AI"),
                String::from("cs.LG"),
                String::from("cs.MA")
            ]
        }
    }

    pub fn new(num_entries: i32, categories: Vec<String>) -> Self {
        Config {
            num_entries: num_entries,
            categories: categories
        }
    }

    pub fn from_env() -> Self {
        dotenvy::from_filename("paperscraper.env").unwrap();

        let num_entries: i32 = env::var("NUM_ENTRIES")
            .expect("NUM_ENTRIES not found in env")
            .parse()
            .unwrap_or_else(|_| {
                eprintln!("Failed to parse NUM_ENTRIES as i32");
                process::exit(1);
            });
        
        let categories: Vec<String> = env::var("CATEGORIES")
            .expect("CATEGORIES not found in env")
            .split_whitespace()
            .map(String::from)
            .collect();

        Self::new(num_entries, categories)
    }
}
