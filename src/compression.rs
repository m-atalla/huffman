use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Root {
    pub left: Box<Option<Node>>,
    pub right: Box<Option<Node>>,
    pub frequency: u32,
}

impl Root {
    pub fn new(frequency: u32, left: Option<Node>, right: Option<Node>) -> Root {
        Root {
            left: Box::new(left),
            right: Box::new(right),
            frequency,
        }
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new(0, None, None)
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
}

/// `BinaryHeap` implementation depends on `Ord` and `PartialOrd` traits
/// for managing how a value is pushed or popped from the internal data structure
/// this implementation flips the order effectively changing the `BinaryHeap`
/// collection from a **max heap** (the default) to a **min heap** (priority queue)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: maybe the following patterns should be replaced with a macro?
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
        if let Some(n1) = prio_queue.pop() {
            if let Some(n2) = prio_queue.pop() {
                // new branch frequency is the sum of
                let new_freq: u32 = n1.variant_freq() + n2.variant_freq();

                // update current root and current frequency
                current_root = Root::new(new_freq, Some(n1), Some(n2));
                current_freq = new_freq;

                // push the new node back into the priority queue
                prio_queue.push(Node::Branch(current_root.clone()));
            }
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

        priority.push(Node::Branch(Root::new(10, None, None)));

        match priority.pop().unwrap() {
            Node::Branch(node) => assert_eq!(node.frequency, 10),
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
}
