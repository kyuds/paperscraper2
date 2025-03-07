use std::{io, env};
use dotenvy;
use async_openai::Client as OpenAIClient;
use aws_config::Region;
use aws_sdk_bedrockruntime::Client as BedrockClient;
use aws_sdk_s3::Client as S3Client;
use paperscraper::{
    agent::{BedrockAgent, OpenAIAgent}, 
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
        process_results(results).await;
    }
    Ok(())
}

async fn process_results(data: Vec<ArxivResult>) {
    dotenvy::from_filename("local.env").unwrap();
    let region = get_env_string("REGION");
    let bucket = get_env_string("BUCKET");

    let conf = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;

    let client = S3Client::new(&conf);
    let s3_storage = S3Storage::new(client, false);

    // Bedrock Agent
    // let client = BedrockClient::new(&conf);
    // let agent = BedrockAgent::new(client);

    // OpenAI API Agent
    let client = OpenAIClient::new();
    let agent = OpenAIAgent::new(client);

    let data = agent.summarize(data).await;
    let key = "local/processed.jsonl";
    let result = s3_storage.upload_arxiv_as_jsonl(
        &bucket, 
        key, 
        &data).await.unwrap();
    println!("{:?}", result);
}

fn get_env_string(key: &str) -> String {
    env::var(key).expect(&format!("{} not found in env", key))
}
