use arxiv::{
    config::Config,
    parser::ArxivParser
};

fn main() {
    let config = Config::from_env();
    let parser = ArxivParser::new(config);
    let results = parser.get_arxiv_results(None);
    println!("# results: {}", results.len());
    println!("{:?}", results[10]);
}
