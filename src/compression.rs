use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Eq, PartialEq)]
pub struct Root {
    pub left: Box<Option<Node>>,
    pub right: Box<Option<Node>>,
    pub frequency: u32,
}

impl Root {
    pub fn new(frequency: u32) -> Root {
        Root {
            left: Box::new(None),
            right: Box::new(None),
            frequency,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Symbol {
    pub value: char,
    pub frequency: u32,
}

impl Symbol {
    pub fn new(value: char, frequency: u32) -> Symbol {
        Symbol { value, frequency }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Branch(Root),
    Leaf(Symbol),
}

impl Node {
    pub fn new_leaf(value: char, frequency: u32) -> Node {
        Node::Leaf(Symbol::new(value, frequency))
    }
}

/// `BinaryHeap` implementation depends on `Ord` and `PartialOrd` traits
/// for managing how a value is pushed or popped from the internal data structure
/// this custom implementation flips the order effectively changing the `BinaryHeap`
/// collect from a **max heap** (the default) to a **min heap** (priority queue)
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

pub fn create_symbol_nodes(frequency_table: &HashMap<char, u32>) -> BinaryHeap<Node> {
    let mut nodes: BinaryHeap<Node> = BinaryHeap::new();

    for (&c, &freq) in frequency_table.iter() {
        nodes.push(Node::new_leaf(c, freq));
    }

    nodes
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn priority_queue_impl_dequeue_order() {
        let mut priority = BinaryHeap::new();

        priority.push(Node::Leaf(Symbol::new('a', 20)));

        priority.push(Node::Branch(Root::new(10)));

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
    fn create_prio_queue_from_frequency_table() {
        let frequency_table: HashMap<char, u32> = HashMap::from([
            ('a', 3),
            ('s', 2),
            ('t', 1)
        ]);

        let mut prio_queue = create_symbol_nodes(&frequency_table);

        // pop (dequeue) should give the minimum value
        match prio_queue.pop().unwrap() {
            Node::Leaf(sym) => assert_eq!(
                sym.frequency,
                *frequency_table.get(&'t').unwrap()
            ),
            _ => (),
        }
    }
}
