use std::{error::Error, fs};
use std::collections::HashMap;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn from_iter(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next()
            .expect("Program name was not included in arguments list..");

        let filename = match args.next() {
            Some(arg) => arg,
            None      => return Err("Didn't get a file name.")
        };

        Ok(Config {
            filename,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;

    println!("Frequency table: {:#?}",frequency(&contents));

    Ok(())
}

// Builds the frequency table for all of the characters of the given
// contents string slice.
fn frequency<'a>(contents: &'a str) -> HashMap<char, u32> {
    let mut frequency_table = HashMap::new();

    for line in contents.lines() {
        for c in line.chars() {
            // Initializes table entry if doesn't exist already
            // dereferences the entry to increment it by once for each occurance
            *frequency_table.entry(c).or_insert(0) += 1;
        }
    }

    frequency_table
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn it_generates_frequency_table() {
       let freq_table = frequency("trust in rust");

       let expected_freq = HashMap::from([
            ('t', 3),
            (' ', 2),
            ('r', 2),
            ('u', 2),
            ('s', 2),
            ('i', 1),
            ('n', 1)
       ]);

       assert_eq!(freq_table, expected_freq);
    }
}
