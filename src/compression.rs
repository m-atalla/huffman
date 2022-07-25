use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Root {
    pub left: Option<Box<Node>>, // 0
    pub right: Option<Box<Node>>, // 1
    pub frequency: u32,
}

impl Root {
    pub fn new(frequency: u32, left: Node, right: Node) -> Root {
        Root {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            frequency,
        }
    }

    pub fn children(self) -> (Option<Box<Node>> , Option<Box<Node>>){
        (self.left, self.right)
    }

}

/// recusively traverses the huffman tree
/// with an 'encoding_path' string that is updated
/// upon going left appends a `0` and going right appends a `1`
/// till it reaches a leaf node at this point, it adds a new entry 
/// to the `encoding_table` **the key** is the character at the current node 
/// and **the value** is the 'encoding_path' to the current node.
pub fn generate_encoding(tree: Root, path: String, mut encoding_table: &mut HashMap<char, String>) {
    if let (Some(left), Some(right)) = tree.children() {
        // TODO: change the following matches to a macro as well?
        match *left {
            Node::Branch(sub_tree) => generate_encoding(sub_tree, path.clone() + "0", &mut encoding_table), 
            Node::Leaf(sym) => {
                encoding_table.insert(sym.value, path.clone() + "0");
            }
        }

        match *right {
            Node::Branch(sub_tree) => generate_encoding(sub_tree, path.clone() + "1", &mut encoding_table), 
            Node::Leaf(sym) => {
                encoding_table.insert(sym.value, path.clone() + "1");
            }
        }
    }
}

impl Default for Root {
    fn default() -> Self {
        Self {
            frequency: 0,
            left: None,
            right: None,
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
}

/// `BinaryHeap` implementation depends on `Ord` and `PartialOrd` traits
/// for managing how a value is pushed or popped from the internal data structure
/// this implementation flips the order effectively changing the `BinaryHeap`
/// collection from a **max heap** (the default) to a **min heap** (priority queue)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: maybe the following patterns/expressions should be replaced with a macro?
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

pub fn create_symbol_nodes_prio_queue(frequency_table: &HashMap<char, u32>) -> BinaryHeap<Node> {
    let mut nodes: BinaryHeap<Node> = BinaryHeap::new();

    for (&c, &freq) in frequency_table.iter() {
        nodes.push(Node::new_leaf(c, freq));
    }

    nodes
}

pub fn create_huffman_tree(mut prio_queue: BinaryHeap<Node>, max_freq: u32) -> Root {
    let mut current_freq = 0u32;
    let mut current_root = Root::default();

    // an empty priority queue should return
    // the default (empty) root
    if prio_queue.len() == 0 {
        return current_root;
    }

    while current_freq < max_freq {
        if let (Some(n1), Some(n2)) = (prio_queue.pop(), prio_queue.pop()) {
            // new branch frequency
            let new_freq: u32 = n1.variant_freq() + n2.variant_freq();

            let (left, right) = n1.cmp_pair(n2);

            // update current root and current frequency
            current_root = Root::new(new_freq, left, right);
            current_freq = new_freq;

            // push the new node back into the priority queue
            prio_queue.push(Node::Branch(current_root.clone()));
        }
    }

    // at this point prio_queue will be dropped
    // since this function takes ownership of the queue
    // and will be cleaned automatically as it goes out of scope.
    current_root
}

#[cfg(test)]
mod test {
    use crate::max_freq;

    use super::*;

    #[test]
    fn priority_queue_impl_dequeue_order() {
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

        let mut prio_queue = create_symbol_nodes_prio_queue(&frequency_table);

        // pop (dequeue) should give the minimum value
        match prio_queue.pop().unwrap() {
            Node::Leaf(sym) => assert_eq!(sym.frequency, *frequency_table.get(&'t').unwrap()),
            _ => (),
        }
    }

    #[test]
    fn it_creates_huffman_tree() {
        let frequency_table: HashMap<char, u32> = HashMap::from([('a', 3), ('s', 2), ('t', 1)]);
        let max_frequency = max_freq(&frequency_table);

        let prio_queue = create_symbol_nodes_prio_queue(&frequency_table);

        let tree = create_huffman_tree(prio_queue, max_frequency);

        assert_eq!(tree.frequency, max_frequency);
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
        let mut encodings = HashMap::new();

        let path = String::default();

        let frequency_table: HashMap<char, u32> = HashMap::from(
            [
                ('d', 5), 
                ('b', 3), 
                ('a', 2),
                ('e', 1)
            ]
        );

        let max_frequency = max_freq(&frequency_table);

        let prio_queue = create_symbol_nodes_prio_queue(&frequency_table);

        let tree = create_huffman_tree(prio_queue, max_frequency);

        generate_encoding(tree, path, &mut encodings);


        for (key, encoding) in &encodings {
            println!("{key} => {encoding}");
        }
    }
}
