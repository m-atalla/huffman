use std::collections::HashMap;
use std::io::{BufReader, BufRead, Error};
use std::fs::File;
use std::path::Path;



macro_rules! some_boxed_leaf {
    ($e:expr) => {
        Some(Box::new(Node::Leaf($e)))
    }
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

#[derive(Debug)]
pub struct Header {
    pub entry_count: u8,
    pub table: HashMap<char, String>,
}

impl Header {
    /// Create header instance from Path 
    /// # Panics 
    /// - The generated huffman table doesn't have as many entries as declared in the first line
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut raw_table = String::default();

        reader.read_line(&mut line)?;

        // remove last byte "\n" (0x0A) byte
        let entry_count = match line[..line.len() -1].parse() {
            Ok(count) => count,
            Err(err) => panic!("invalid file format!\nParseIntError: {}", err),
        };


        for _ in 0..entry_count {
            let mut buf_line = String::new();

            reader.read_line(&mut buf_line).unwrap();

            raw_table.push_str(&buf_line);
        }

        let table = Header::parse_huffman_table(&raw_table);

        // length of the generated table should be equal to
        // the header `entry_count`
        assert!(table.len() as u8 == entry_count);
        
        
        let root = Root::from_table(&table);

        println!("{root:#?}");



        Ok(Self {
            entry_count,
            table
        })
    }

    pub fn parse_huffman_table(raw: &str) -> HashMap<char, String> {
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
                match  ch {
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
    pub fn walk(root: &Root, code_elem: char) -> Box<Node> {
        match code_elem {
            '1' => walk!(&root.right),
            '0' => walk!(&root.left),
            other_char => invalid_code!(other_char)
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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_huffman_table() {
        let table_str = String::from("a01\nc001");
        let table = Header::parse_huffman_table(&table_str);
        assert_eq!(table.get(&'a').unwrap(), "01");
        assert_eq!(table.get(&'c').unwrap(), "001");
    }

    #[test]
    fn huffman_table_parser_handles_newlines() {
        let table_str = String::from("\\n01\n");

        let table = Header::parse_huffman_table(&table_str);

        assert_eq!(table.get(&'\n').unwrap(), "01");
    }

    #[test]
    fn reconstruct_huffman_from_table() {
        let table = HashMap::from([
            ('x', "0".to_string()),
            ('y', "11".to_string()),
            ('z', "10".to_string()),
        ]);

        let tree = Root::from_table(&table);

        let y = tree.right.clone()
            .unwrap()
            .branch()
            .unwrap()
            .right
            .unwrap()
            .leaf()
            .unwrap()
            .value;


        let z = tree.right.clone()
            .unwrap()
            .branch()
            .unwrap()
            .left
            .unwrap()
            .leaf()
            .unwrap()
            .value;
        
        let x = tree.left
            .unwrap()
            .leaf()
            .unwrap()
            .value;

        assert_eq!(y, 'y');
        assert_eq!(z, 'z');
        assert_eq!(x, 'x');
    }
}
