use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use chrono::{Utc, Duration};
use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};
use serde_json::Value;

use paperscraper2::{
    storage::S3Storage,
    parser::ArxivParser,
    config::Config
};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

// when testing, lambda functions cannot accept LambdaEvent<()>
async fn func(_event: LambdaEvent<Value>) -> Result<(), LambdaError> {
    let parser_config = Config::default();
    let offset = parser_config.date_offset;
    let parser = ArxivParser::from_config(parser_config);
    let data = parser.get_arxiv_results(None).await;

    let region = get_env_string("REGION");
    let bucket = get_env_string("BUCKET");
    let key = get_s3_key(offset as i64);

    let conf = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;
    let client = S3Client::new(&conf);
    let s3_storage = S3Storage::default(client);

    let _result = s3_storage.upload_raw_arxiv_as_jsonl(
        bucket.as_str(), 
        key.as_str(), 
        &data).await?;

    Ok(())
}

fn get_env_string(key: &str) -> String {
    std::env::var(key).expect(format!("{} not found in env", key).as_str())
}

fn get_s3_key(offset: i64) -> String {
    let curr = Utc::now();
    let d0 = curr - Duration::days(offset + 1);
    let d1 = curr - Duration::days(offset);
    format!("raw/{}-{}.jsonl", d0.format("%y%m%d"), d1.format("%y%m%d"))
}
