use std::collections::HashMap;
use std::io::{BufReader, BufRead, Error, Write};
use std::fs::{self, File};
use crate::Config;
use bitvec::prelude::*;


pub fn decompress(config: &Config) -> Result<(), Error>{
    // Open file and create a buffered reader.
    let file = File::open(config.get_input_file())?;

    let mut reader = BufReader::new(file);

    let reconst = parse_header(&mut reader)?;

    write_decoded(config,&mut reader, reconst)?;

    Ok(())
}

fn write_decoded(config: &Config,reader: &mut BufReader<File>, reconst: Reconst) -> Result<(), Error> {
    let mut encoded_content: Vec<u8> = Vec::new();

    reader.read_until(b'\0', &mut encoded_content)?;

    let bv: BitVec<u8, Lsb0> = BitVec::from_vec(encoded_content);

    let decoded = tread(&reconst.huffman_tree, bv);

    let output_path = config.get_output_file()?;

    let mut output_file = fs::OpenOptions::new()
        .write(true)
        .open(output_path)?;

    output_file.write_all(decoded.as_bytes())?;

    Ok(())
}

fn parse_header(reader: &mut BufReader<File>) -> Result<Reconst, Error>{
    let mut raw_table = String::new();

    let mut line = String::new();

    reader.read_line(&mut line)?;

    // remove last byte 0x0A (\n)
    line.pop();

    let entry_count: u8 = match line.parse() {
        Ok(count) => count,
        Err(err) => {
            panic!(
                "Invalid file format! \
                expected first line to be a number, found: `{}`\
                \nParseIntError: {}",
                line,
                err
            );
        },
    };

    for _ in 0..entry_count {
        let mut buf_line = String::new();

        reader.read_line(&mut buf_line)?;

        raw_table.push_str(&buf_line);
    }

    Ok(Reconst::from_str(entry_count, &raw_table))
}

#[derive(Debug)]
pub struct Reconst {
    pub encoding_table: HashMap<char, String>,
    pub huffman_tree: Root
}

impl Reconst {
    /// Create header instance from Path 
    /// # Panics 
    /// - The generated huffman table doesn't have as many entries as declared in the first line
    pub fn from_str(entry_count: u8, raw_table: &str) -> Self{
        let encoding_table = Reconst::huffman_table(&raw_table);

        // length of the generated table should be equal to
        // the header `entry_count`
        assert!(encoding_table.len() as u8 == entry_count);
        
        let huffman_tree = Root::from_table(&encoding_table);

        Self {
            encoding_table,
            huffman_tree
        }
    }

    pub fn huffman_table(raw: &str) -> HashMap<char, String> {
        let mut table = HashMap::new();
        for line in raw.lines() {
            let key = match line.chars().next() {
                // Corrects escaped newline character iteraction 
                // with .lines() iterator
                Some(k) => if k == '\\' { '\n' } else { k },
                None => break
            };

            let code = if key == '\n' {
                // Since new lines are escaped by adding an extra escape character..
                // actual code value start is shifted one index
                String::from(&line[2..])
            } else {
                String::from(&line[1..])
            };

            table.insert(key, code);
        }
        table
    }
}

macro_rules! some_boxed_leaf {
    ($e:expr) => {
        Some(Box::new(Node::Leaf($e)))
    };
}

macro_rules! some_boxed_branch {
    ($e:expr) => {
        Some(Box::new(Node::Branch($e)))
    };
}

macro_rules! extend {
    ($root:expr, $code:expr, $symbol_value:expr) => {
        match &$root {
            Some(tree) => {
                let sub_root = tree.clone().branch().unwrap_or_else(||
                    panic!("Failed to extend branch out of sub tree {:?}", tree)
                );

                $root = some_boxed_branch!(
                    Root::new_traverse(Some(sub_root), &$code[1..], $symbol_value)
                );
            },
            None => {
                $root = some_boxed_branch!(
                    Root::new_traverse(None, &$code[1..], $symbol_value)
                );
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct Root {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Default for Root {
    fn default() -> Self {
        Self {
            left: None,
            right: None
        }
    }
}

macro_rules! walk {
    ($sub_tree:expr) => {
        match $sub_tree{
            Some(node) => node.clone(),
            None => panic!("Invalid code or root was provided.")
        }
    };
}

macro_rules! invalid_code {
    ($invalid_char:expr) => {
       panic!("Invalid code expected a `0` or `1`, got `{}`", $invalid_char)
    };
}

impl Root {
    pub fn from_table(table: &HashMap<char, String>) -> Self {
        table.iter().fold(Root::default(), |acc, (key, value)| { 
            Root::new_traverse(Some(acc), value, *key) 
        })
    }

    /// Reconstructs the huffman tree through traversal of header code strings
    /// # Panics
    /// - On getting an invalid code other than a `0` or `1`
    /// - On getting an empty string.
    /// - On overwriting node leaf variant.
    pub fn new_traverse(bootstrap: Option<Root>, code: &str, symbol_value: char) -> Self {
        // Use the bootstrap root if it's provided.
        let mut root = if let Some(_root) = bootstrap { 
            _root 
        } else { 
            Self::default()
        };

        // consume string slice until reaching the last char
        // (base condition)
        if code.len() == 1 {
            // its safe to unwrap here already explicitly
            // checked that the str slice has a single character
            let ch = code.chars().nth(0).unwrap();

            match ch {
                '1' => root.right = some_boxed_leaf!(Symbol::new(symbol_value)),
                '0' => root.left = some_boxed_leaf!(Symbol::new(symbol_value)),
                other_char => invalid_code!(other_char)
            };

            return root;
        } 

        match code.chars().next() {
            Some(ch) => {
                match ch {
                    '1' => extend!(root.right, code, symbol_value),
                    '0' => extend!(root.left, code, symbol_value),
                    other_char => invalid_code!(other_char) 
                }
            },
            None => panic!("Failed to traverse tree with, got empty code string.")
        };

        root
    } 


    /// Incremental tree traversal a tree given a char (code fragment)
    /// # Panics
    /// - On receiving a code that isn't a `0` or `1`
    /// - On providing an empty code slice.
    /// - On providing an invalid root provided (tree was reconstructed incorrectly).
    pub fn walk(root: &Root, code_elem: bool) -> Box<Node> {
        match code_elem {
            true => walk!(&root.right),
            false => walk!(&root.left),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Branch(Root),
    Leaf(Symbol)
}

impl Node {
    pub fn branch(self) -> Option<Root> {
        match self {
            Node::Branch(root) => Some(root),
            Node::Leaf(_) => None,
        }
    }

    #[cfg(test)]
    pub fn leaf(self) -> Option<Symbol> {
        match self {
            Node::Leaf(leaf) => Some(leaf),
            Node::Branch(_) => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Symbol {
    pub value: char
}

impl Symbol {
    fn new(value: char) -> Self {
        Self {
            value,
        }
    }
}

/// Incrementally walks the huffman tree using the provided code slice
/// and returns a decoded string.
/// # Panics:
/// - `Root::walk` panic conditions
pub fn tread(huffman_tree: &Root, code_path: BitVec<u8, Lsb0>) -> String {
    let mut decoded = String::new();
    let mut walk_root: Option<Root> = None;

    for code in code_path {
        let leg = match &walk_root {
            Some(_leg) => _leg,
            None => huffman_tree
        };

        match *Root::walk(leg, code) {
            Node::Leaf(symbol) => {
                walk_root = None;
                decoded.push(symbol.value);
            },
            Node::Branch(root) => {
                walk_root = Some(root)
            },
        };
    }

    decoded
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_huffman_table() {
        let table_str = String::from("a01\nc001");
        let table = Reconst::huffman_table(&table_str);
        assert_eq!(table.get(&'a').unwrap(), "01");
        assert_eq!(table.get(&'c').unwrap(), "001");
    }

    #[test]
    fn huffman_table_parser_handles_newlines() {
        let table_str = String::from("\\n01\n");

        let table = Reconst::huffman_table(&table_str);

        assert_eq!(table.get(&'\n').unwrap(), "01");
    }


    fn basic_tree() -> Root {
        let table = HashMap::from([
            ('h', "010".to_string()),
            ('f', "11".to_string()),
            ('m', "011".to_string()),
            ('n', "000".to_string()),
            ('a', "100".to_string()),
            ('u', "001".to_string()),
            ('\n', "101".to_string()),
        ]);

        Root::from_table(&table)
    }

    #[test]
    fn reconstruct_huffman_from_table() {
        let tree = basic_tree();

        let f = tree.right
            .unwrap()
            .branch()
            .unwrap()
            .right
            .unwrap()
            .leaf()
            .unwrap();

        assert_eq!(f.value, 'f');
    }

    #[test]
    fn huffman_tree_decode_walk() {
        let tree = basic_tree();
        let step_1 = Root::walk(&tree, false).branch().unwrap();
        let step_2 = Root::walk(&step_1, true).branch().unwrap();
        let step_3 = Root::walk(&step_2, false).leaf().unwrap();
        assert_eq!(step_3.value, 'h');

        let step_1 = Root::walk(&tree, false).branch().unwrap();
        let step_2 = Root::walk(&step_1, false).branch().unwrap();
        let step_3 = Root::walk(&step_2, false).leaf().unwrap();
        assert_eq!(step_3.value, 'n');

        let step_1 = Root::walk(&tree, true).branch().unwrap();
        let step_2 = Root::walk(&step_1, false).branch().unwrap();
        let step_3 = Root::walk(&step_2, true).leaf().unwrap();
        assert_eq!(step_3.value, '\n');
    }
    

    #[test]
    fn huffman_tree_decode_tread_path() {
        let tree = basic_tree();
        let decomp = tread(&tree, bitvec![u8, Lsb0; 0,1,0]);
        assert_eq!(&decomp, "h");
    }
}
