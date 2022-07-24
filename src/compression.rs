use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Branch(Root),
    Leaf(Symbol),
}

pub trait Freq {
    fn frequency(&self) -> u32;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Root {
    pub left: Box<Option<Node>>,
    pub right: Box<Option<Node>>,
    pub frequency: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Symbol {
    pub value: char,
    pub frequency: u32,
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
                Node::Branch(other_node) =>  other_node.frequency.cmp(&node.frequency),
                Node::Leaf(other_node) => other_node.frequency.cmp(&node.frequency),
            },
            Node::Leaf(node) => match other {
                Node::Branch(other_node) => other_node.frequency.cmp(&node.frequency),
                Node::Leaf(other_node) => other_node.frequency.cmp(&node.frequency)
            }
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

impl Symbol {
    pub fn new(frequency: u32, value: char) -> Symbol {
        Symbol { value, frequency }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn priority_queue_impl_dequeue_order() {
        let mut priority = BinaryHeap::new();


        priority.push(
            Node::Leaf(Symbol::new(20, 'a'))
        );

        priority.push(
            Node::Branch(Root::new(10))
        );


        match priority.pop().unwrap() {
            Node::Branch(node) => assert_eq!(node.frequency, 10),
            _ => (), 
        };

        match priority.pop().unwrap() {
            Node::Leaf(node) => {
                assert_eq!(node.frequency, 20);
                assert_eq!(node.value, 'a');
            },
            _ => (),
        };
    }
}
