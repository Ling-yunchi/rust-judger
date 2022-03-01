mod communicate;
mod config;
mod engine;
mod runner;

use std::env;

use config::Config;

use crate::engine::run;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    println!("{:?}", config);
    run(config).await.unwrap_or_else(|err| {
        eprintln!("Problem running engine: {}", err);
        std::process::exit(1);
    });
}
