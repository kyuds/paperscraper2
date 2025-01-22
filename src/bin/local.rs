use std::io;

use paperscraper2::{
    config::Config,
    parser::{ArxivParser, ArxivResult},
    storage::LocalSaver
};

#[tokio::main]
async fn main() -> io::Result<()> {
    // get arxiv data
    let config = Config::from_env();
    let parser = ArxivParser::from_config(config);
    let results = parser.get_arxiv_results(None);
    println!("# results: {}", results.len());
    
    // write arxiv data to local storage and AWS S3
    write_results_local(&results)?;
    // write_results_s3(&results).await;

    Ok(())
}

fn write_results_local(data: &Vec<ArxivResult>) -> io::Result<()> {
    LocalSaver::save_raw_arxiv_results_as_readme("arxiv.md", data)?;
    LocalSaver::save_raw_arxiv_as_jsonl("arxiv.jsonl", data)?;
    Ok(())
}

async fn write_results_s3(data: &Vec<ArxivResult>) {

}
