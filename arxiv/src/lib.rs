mod config;

use config::Config;

pub fn print_env() {
    let conf = Config::from_env();
    println!("{:?}", conf);
}
