use std::env;
use std::process;
use huffman::Config;

fn main() {
    let config = Config::from_iter(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("file name = {}", config.filename);

    if let Err(e) = huffman::run(config) {
        eprintln!("Compression error: {}", e);

        process::exit(1);
    }
}
