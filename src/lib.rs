use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::num::ParseIntError;
use std::path::PathBuf;

pub mod encode;
pub mod decode;

pub struct Config {
    pub input_file: String,
    pub output_file: Option<String>,
    pub mode: Mode,
}

#[derive(Debug)]
pub enum Mode {
    Compress,
    Decompress
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Compress => write!(f, "Compression"),
            Mode::Decompress => write!(f, "Decompression")
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_file: String::default(),
            output_file: None,
            mode: Mode::Compress,
        }
    }
}


impl Config {
    /// Parse Config from args iterator
    /// # Panics
    /// - Empty args iterator
    /// - No output file name provided after '-o' flag (next is a flag or next is empty)
    pub fn from_iter(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next()
            .expect("Program name was not included in arguments list.");

        let mut config = Config::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-d" => config.mode = Mode::Decompress,
                "-o" => {
                    config.output_file = match args.next() {
                        Some(param) => {
                            if param.starts_with('-') {
                                panic!("Expected filename argument got a flag the flag '{param}' instead.");
                            }
                            Some(param)
                        },
                        None => {
                            return Err("Expected an output file name after '-o' flag.");
                        }
                    };
                },
                input_file => config.input_file = input_file.to_string()
            }
        }

        Ok(config)
    }
}

// ignored for now...
// need some sort of a "BitWriter" implementation
// to represent bits efficiently
#[ignore = "dead_code"]
pub fn table_bits(table: &HashMap<char, String>) -> Result<HashMap<char, u8>, ParseIntError> {

    let mut new_map = HashMap::new();

    for (k, v) in table.iter() {
        let bin = i8::from_str_radix(v, 2)?;
        new_map.insert(*k, bin as u8);
    }

    Ok(new_map)
}

fn create_output_file(config: &Config) -> Result<PathBuf, Box<dyn Error>> {
    // setup path
    let out_filename = match config.output_file.clone() {
        Some(name) => name,
        None => {
            config.input_file.clone() + ".o"
        },
    };

    let path_buf = PathBuf::from(&out_filename);

    if !path_buf.exists() {
        File::create(&path_buf)?;
    }

    Ok(path_buf)
}

fn compress(config: &Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.input_file.clone())?;

    let table = encode::generate_encoding_table(&contents);


    let out_path = create_output_file(&config)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(out_path)?;

    // writing header
    let mut head_buf = format!("{}\n", table.len())
        .as_bytes()
        .to_owned();

    for (k, v) in &table {
        let line_buf = if *k == '\n' {
            format!("{}{v}\n", "\\n").as_bytes().to_owned()
        } else {
            format!("{k}{v}\n").as_bytes().to_owned()
        };

        head_buf.extend(line_buf);
    }

    file.write(&head_buf)?;

    for sym in contents.chars() {
        match table.get(&sym) {
            Some(bin) => file.write(bin.as_bytes())?,
            None => continue
        };
    }

    Ok(())
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {

    match config.mode {
        Mode::Compress => compress(&config)?,
        Mode::Decompress => println!("Decompressing or something lol")
    }
    // let header = decode::Header::from(out_path)?;
    // println!("{header:#?}");

    Ok(())
}
