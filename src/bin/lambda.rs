use async_openai::Client as OpenAIClient;
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};
use serde_json::Value;

use paperscraper::{
    agent::OpenAIAgent,
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
    let region = get_env_string("REGION");
    let bucket = get_env_string("BUCKET");

    let parser_config = ArxivConfig::default();
    let name_config = NameConfig::default(&bucket);
    let parser = ArxivParser::from_config(parser_config);
    let data = parser.get_arxiv_results(None).await;

    if data.is_empty() {
        println!("No results. Exiting...");
        return Ok(());
    }
    
    let conf = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;
    let s3_client = S3Client::new(&conf);
    let s3_storage = S3Storage::default(s3_client);
    let key = name_config.raw_jsonl_path();
    let _ = s3_storage.upload_arxiv_as_jsonl(
        &bucket, 
        &key, 
        &data).await?;
    let openai_client = OpenAIClient::new();
    let agent = OpenAIAgent::new(openai_client);
    let data = agent.summarize(data).await;
    let key = name_config.processed_jsonl_path();
    let _ = s3_storage.upload_arxiv_as_jsonl(
        &bucket, 
        &key, 
        &data).await?;
    Ok(())
}

fn get_env_string(key: &str) -> String {
    std::env::var(key).expect(&format!("{} not found in env", key))
}
