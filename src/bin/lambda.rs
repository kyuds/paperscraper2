use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};

use paperscraper2::parser::ArxivParser;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_event: LambdaEvent<()>) -> Result<(), LambdaError> {
    let parser = ArxivParser::new();
    let arxiv_data = parser.get_arxiv_results(None);
    Ok(())
}
