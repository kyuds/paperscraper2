use std::{io, env};
use dotenvy;
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use paperscraper2::{
    config::Config,
    parser::{ArxivParser, ArxivResult},
    storage::{LocalSaver, S3Saver}
};

#[tokio::main]
async fn main() -> io::Result<()> {
    // get arxiv data
    let config = Config::from_env();
    let parser = ArxivParser::from_config(config);
    let results = parser.get_arxiv_results(None).await;
    println!("# results: {}", results.len());
    
    // // write arxiv data to local storage and AWS S3
    write_results_local(&results)?;
    write_results_s3(&results).await;

    Ok(())
}

fn write_results_local(data: &Vec<ArxivResult>) -> io::Result<()> {
    LocalSaver::save_raw_arxiv_results_as_readme("arxiv.md", data)?;
    LocalSaver::save_raw_arxiv_as_jsonl("arxiv.jsonl", data)?;
    Ok(())
}

async fn write_results_s3(data: &Vec<ArxivResult>) {
    dotenvy::from_filename("local_aws.env").unwrap();
    let region = get_env_string("REGION");
    let bucket = get_env_string("BUCKET");

    let conf = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;

    let client = S3Client::new(&conf);
    let key = "raw/arxiv.jsonl";
    let result = S3Saver::upload_raw_arxiv_as_jsonl(
        &client, 
        bucket.as_str(), 
        key, 
        &data).await.unwrap();
    println!("{:?}", result);
}

fn get_env_string(key: &str) -> String {
    env::var(key).expect(format!("{} not found in env", key).as_str())
}
