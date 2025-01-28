use std::{io, env};
use dotenvy;
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use paperscraper2::{
    config::ArxivConfig,
    model::ArxivResult,
    parser::ArxivParser,
    storage::S3Storage
};

#[tokio::main]
async fn main() -> io::Result<()> {
    // get arxiv data
    let config = ArxivConfig::from_env();
    let parser = ArxivParser::from_config(config);
    let results = parser.get_arxiv_results(None).await;
    println!("# results: {}", results.len());
    if results.len() > 0 {
        // write arxiv data to AWS S3
        write_results_s3(&results).await;
    }
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
    let s3_storage = S3Storage::new(client, false);

    let key = "local/arxiv.jsonl";
    let result = s3_storage.upload_raw_arxiv_as_jsonl(
        bucket.as_str(), 
        key, 
        &data).await.unwrap();
    println!("{:?}", result);

    let result = s3_storage.upload_bedrock_inputs(
        bucket.as_str(), 
        key, 
        &data).await.unwrap();
    println!("{:?}", result);
}

fn get_env_string(key: &str) -> String {
    env::var(key).expect(format!("{} not found in env", key).as_str())
}
