use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs;
use std::io::Write;
use std::error::Error;
use crate::Config;
use bitvec::prelude::*;


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Root {
    pub left: Box<Node>, // 0
    pub right: Box<Node>, // 1
    pub frequency: u32,
}

impl Root {
    pub fn new(frequency: u32, left: Node, right: Node) -> Root {
        Root {
            left: Box::new(left),
            right: Box::new(right),
            frequency,
        }
    }

    #[inline]
    pub fn children(self) -> (Box<Node> , Box<Node>){
        (self.left, self.right)
    }

}

impl Default for Root {
    fn default() -> Self {
        Self {
            frequency: 0,
            left: Box::new(Node::Leaf(Symbol::default())),
            right: Box::new(Node::Leaf(Symbol::default())),
        }
    }
}


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Symbol {
    pub value: char,
    pub frequency: u32,
}

impl Symbol {
    pub fn new(value: char, frequency: u32) -> Symbol {
        Symbol { value, frequency }
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            frequency: 0,
            value: '_'
        }
    }
}

macro_rules! encode_child {
    ($child_node:expr, $suffix_code: expr, $path:expr, $table:expr) => {
        {
            let mut _path_vec = $path.clone(); 

            _path_vec.push($suffix_code);

            match &*$child_node {
                Node::Leaf(sym) => {
                    $table.insert(sym.value, _path_vec);
                },
                sub_tree => sub_tree.generate_encoding(_path_vec, &mut $table), 
            };
        }
    };
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Node {
    Branch(Root),
    Leaf(Symbol),
}

impl Node {
    pub fn new_leaf(value: char, frequency: u32) -> Node {
        Node::Leaf(Symbol::new(value, frequency))
    }

    pub fn variant_freq(&self) -> &u32 {
        match self {
            Node::Branch(root) => &root.frequency,
            Node::Leaf(sym) => &sym.frequency,
        }
    }

    /// compares the current node with another and returns a sorted in a pair tuple
    ///
    /// for **pattern matching** the pair tuple:
    ///  - the smaller node on the left (index 0)
    ///  - the bigger node on the right (index 1)
    pub fn cmp_pair(self, other: Node) -> (Node, Node){
        if self.variant_freq() < other.variant_freq() {
            (self, other)
        } else {
            (other, self)
        }
    }

    /// recusively traverses the huffman tree
    /// with an 'encoding_path' string that is updated
    /// upon going left appends a `0` and going right appends a `1`
    /// till it reaches a leaf node at this point, it adds a new entry 
    /// to the `encoding_table` **the key** is the character at the current node 
    /// and **the value** is the 'encoding_path' to the current node.
    /// # Panics:
    /// - Running into a 'Node Leaf' variant
    pub fn generate_encoding(&self, path: BitVec<u8, Msb0>, mut encoding_table: &mut HashMap<char, BitVec<u8, Msb0>>) {
        match self {
            Node::Branch(root) => {
                encode_child!(root.left, false, path, encoding_table);
                encode_child!(root.right, true, path, encoding_table);
            }
            Node::Leaf(_) => {
                panic!("Expected a `Node::Branch` variant got a `Node::Leaf`");
            }
        }
    }
}

/// `BinaryHeap` implementation depends on `Ord` and `PartialOrd` traits
/// for managing how a value is pushed or popped from the internal data structure
/// this implementation flips the order effectively changing the `BinaryHeap`
/// collection from a **max heap** (the default) to a **min heap** (priority queue)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Node::Branch(node) => match other {
                Node::Branch(other_node) => other_node.frequency.cmp(&node.frequency),
                Node::Leaf(other_node) => other_node.frequency.cmp(&node.frequency),
            },
            Node::Leaf(node) => match other {
                Node::Branch(other_node) => other_node.frequency.cmp(&node.frequency),
                Node::Leaf(other_node) => other_node.frequency.cmp(&node.frequency),
            },
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn init_symbol_nodes_prio_queue(frequency_table: &HashMap<char, u32>) -> BinaryHeap<Node> {
    let mut nodes: BinaryHeap<Node> = BinaryHeap::new();

    for (&c, &freq) in frequency_table.iter() {
        nodes.push(Node::new_leaf(c, freq));
    }

    nodes
}

pub fn create_huffman_tree(mut prio_queue: BinaryHeap<Node>) -> Node {
    if prio_queue.len() == 0 {
        panic!("Empty priority queue..aborting");
    }

    while prio_queue.len() > 1 {
        if let (Some(n1), Some(n2)) = (prio_queue.pop(), prio_queue.pop()) {
            // new branch frequency
            let new_freq: u32 = n1.variant_freq() + n2.variant_freq();

            let (left, right) = n1.cmp_pair(n2);

            // push the new node back into the priority queue
            prio_queue.push(
                Node::Branch(
                    Root::new(new_freq, left, right)
                )
            );
        }
    }

    // at this point prio_queue will be dropped
    // since this function takes ownership of the queue
    // and will be cleaned automatically as it goes out of scope.
    prio_queue.pop().unwrap()
}

pub fn generate_encoding_table(contents: &str) -> HashMap<char, BitVec<u8, Msb0>>{
    let frequency_table = init_frequency_table(&contents);

    let path = bitvec!(u8, Msb0;);

    let mut encoding_table = HashMap::new();

    let prio_queue = init_symbol_nodes_prio_queue(&frequency_table);

    let tree = create_huffman_tree(prio_queue);

    tree.generate_encoding(path, &mut encoding_table);

    encoding_table
}

// Builds the frequency table for all of the characters of the given
// contents string slice.
fn init_frequency_table(contents: &str) -> HashMap<char, u32> {
    let mut frequency_table = HashMap::new();
    for sym in contents.chars() {
        // Initializes table entry if doesn't exist
        // dereferences the entry to increment it by one for each occurance
        *frequency_table.entry(sym).or_insert(0) += 1;
    }

    frequency_table
}

pub fn compress(config: &Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.input_file.clone())?;

    let table = generate_encoding_table(&contents);

    let out_path = config.get_output_file()?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(out_path)?;

    // writing header
    let head_buf = format!("{}\n", table.len())
        .as_bytes()
        .to_owned();

    file.write(&head_buf)?;


    let mut table_buf = Vec::new();

    for (symbol, bits) in &table {

        let code = fmt_bitvec(bits);

        let line_buf = if *symbol == '\n' {
            format!("{}{}\n", "\\n", code).as_bytes().to_owned()
        } else {
            format!("{}{}\n", symbol, code).as_bytes().to_owned()
        };

        table_buf.extend(line_buf);
    }

    file.write(&table_buf)?;


    let mut bit_vec_buff = bitvec!();
    let mut vec_buf = vec![];
    for sym in contents.chars() {
        match table.get(&sym) {
            Some(bin) => bit_vec_buff.extend(bin),
            None => continue
        };
    }

    for chunk in bit_vec_buff.chunks(8) {
        vec_buf.push(chunk.load::<u8>());
    }
    file.write_all(&vec_buf)?;

    Ok(())
}

fn fmt_bitvec(bits: &BitSlice<u8, Msb0>) -> String {
    let mut code = String::new();
    for bit in bits.iter().by_vals() {
        match bit {
            true => code.push('1'),
            false => code.push('0')
        }
    }

    code
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_generates_frequency_table() {
        let freq_table = init_frequency_table("huffman");

        let expected_freq =
            HashMap::from([('h', 1), ('u', 1), ('f', 2), ('m', 1), ('a', 1), ('n', 1)]);

        assert_eq!(freq_table, expected_freq);
    }

    #[test]
    fn min_heap_impl() {
        let mut priority = BinaryHeap::new();

        priority.push(Node::Leaf(Symbol::new('a', 20)));

        priority.push(Node::Branch(Root::default()));

        match priority.pop().unwrap() {
            Node::Branch(node) => assert_eq!(node.frequency, 0),
            _ => (),
        };

        match priority.pop().unwrap() {
            Node::Leaf(node) => {
                assert_eq!(node.frequency, 20);
                assert_eq!(node.value, 'a');
            }
            _ => (),
        };
    }

    #[test]
    fn it_creates_prio_queue_from_frequency_table() {
        let frequency_table: HashMap<char, u32> = HashMap::from([('a', 3), ('s', 2), ('t', 1)]);

        let mut prio_queue = init_symbol_nodes_prio_queue(&frequency_table);

        // pop (dequeue) should give the minimum value
        match prio_queue.pop().unwrap() {
            Node::Leaf(sym) => assert_eq!(sym.frequency, *frequency_table.get(&'t').unwrap()),
            _ => (),
        }
    }

    #[test]
    fn it_creates_huffman_tree() {
        let frequency_table: HashMap<char, u32> = HashMap::from([('a', 3), ('s', 2), ('t', 1)]);

        let prio_queue = init_symbol_nodes_prio_queue(&frequency_table);

        let tree = create_huffman_tree(prio_queue);

        let max_frequency: u32 = frequency_table.values().sum();

        // the root of the generated huffman tree should be equal to the sum of values
        // in the huffman table.
        assert_eq!(*tree.variant_freq(), max_frequency);
    }

    #[test]
    fn it_sorts_node_pair() {
        let mut r1 = Root::default();
        let mut r2 = Root::default();

        r1.frequency = 20;
        r2.frequency = 10;

        let n1 = Node::Branch(r1);
        let n2 = Node::Branch(r2);

        if let (Node::Branch(s1), Node::Branch(s2)) = n1.cmp_pair(n2) {
            assert_eq!(s1.frequency, 10);
            assert_eq!(s2.frequency, 20);
        }
    }

    #[test]
    fn it_generates_correct_encoding() {
        let txt = "dddddbbbaae";
        
        let encoding_table = generate_encoding_table(txt);

        let code = encoding_table.get(&'d').unwrap();

        assert_eq!(code, &bitvec![u8, Msb0; 0]);
    }
}
