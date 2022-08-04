use huffman::Config;
use std::env;
use std::process;

fn main() {
    let config = Config::from_iter(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = huffman::run(&config) {
        eprintln!("{} error: {}", config.mode, e);

        process::exit(1);
    }
}
