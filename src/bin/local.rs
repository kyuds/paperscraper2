use std::io;

use paperscraper2::{
    config::Config,
    parser::ArxivParser,
    utils::LocalSaver
};

fn main() -> io::Result<()> {
    let config = Config::from_env();
    let parser = ArxivParser::from_config(config);
    let results = parser.get_arxiv_results(None);
    println!("# results: {}", results.len());
    LocalSaver::save_raw_arxiv_results_as_readme("arxiv.md", &results)?;
    Ok(())
}
