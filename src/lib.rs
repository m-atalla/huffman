use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
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
                                panic!("Expected filename argument got the flag '{param}' instead.");
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

    pub fn get_output_file(&self) -> Result<PathBuf, Box<dyn Error + 'static>> {
        let out_filename = match self.output_file.clone() {
            Some(name) => name,
            None => self.input_file.clone() + ".o"
        };

        let path_buf = PathBuf::from(out_filename);

        if !path_buf.exists() {
            File::create(&path_buf)?;
        }

        Ok(path_buf)
    }

    #[inline(always)]
    pub fn get_input_file(&self) -> PathBuf {
        PathBuf::from(&self.input_file)
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



pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.mode {
        Mode::Compress => encode::compress(&config)?,
        Mode::Decompress => decode::decompress(&config)?,
    }

    Ok(())
}
