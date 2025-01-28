use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};
use serde_json::Value;

use paperscraper2::{
    config::{ArxivConfig, NameConfig}, 
    parser::ArxivParser, 
    storage::S3Storage
};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

// when testing, lambda functions cannot accept LambdaEvent<()>
async fn func(_event: LambdaEvent<Value>) -> Result<(), LambdaError> {
    let parser_config = ArxivConfig::default();
    let name_config = NameConfig::default();
    let parser = ArxivParser::from_config(parser_config);
    let data = parser.get_arxiv_results(None).await;

    let region = get_env_string("REGION");
    let bucket = get_env_string("BUCKET");

    let conf = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;
    let s3_client = S3Client::new(&conf);
    let s3_storage = S3Storage::default(s3_client);

    let _ = s3_storage.upload_raw_arxiv_as_jsonl(
        bucket.as_str(), 
        &name_config.raw_jsonl_path(), 
        &data).await?;



    Ok(())
}

fn get_env_string(key: &str) -> String {
    std::env::var(key).expect(format!("{} not found in env", key).as_str())
}

