use std::collections::HashMap;
use std::{error::Error, fs};

pub mod compression;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn from_iter(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next()
            .expect("Program name was not included in arguments list..");

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file name."),
        };

        Ok(Config { filename })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;

    println!("Frequency table: {:#?}", frequency(&contents));

    Ok(())
}

// Builds the frequency table for all of the characters of the given
// contents string slice.
fn frequency(contents: &str) -> HashMap<char, u32> {
    let mut frequency_table = HashMap::new();

    for line in contents.lines() {
        for c in line.chars() {
            // Initializes table entry if doesn't exist
            // dereferences the entry to increment it by one for each occurance
            *frequency_table.entry(c).or_insert(0) += 1;
        }
    }

    frequency_table
}

pub fn max_freq(frequency_table: &HashMap<char, u32>) -> u32 {
    frequency_table.values().sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_generates_frequency_table() {
        let freq_table = frequency("huffman");

        let expected_freq =
            HashMap::from([('h', 1), ('u', 1), ('f', 2), ('m', 1), ('a', 1), ('n', 1)]);

        assert_eq!(freq_table, expected_freq);
    }

    #[test]
    fn it_calculates_the_maximum_frequency() {
        let freq_table = frequency("abcdefffgggabc");

        let max = max_freq(&freq_table);

        assert_eq!(max, 14);
    }
}
