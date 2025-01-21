use arxiv::{
    config::Config,
    parser::ArxivParser
};

fn main() {
    let config = Config::from_env();
    let parser = ArxivParser::new(config);
    println!("{:?}", parser.get_arxiv_results(None));
}
