use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Branch(Root),
    Leaf(Symbol),
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

macro_rules! impl_min_heap {
    (for $($t:ty),+) => {
        $(impl Ord for $t {
            fn cmp(&self, other: &Self) -> Ordering {
                other.frequency.cmp(&self.frequency)
            }
        }
        impl PartialOrd for $t {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        })*
    }
}

impl_min_heap!(for Root, Symbol);

impl Root {
    pub fn new(frequency: u32) -> Root {
        Root {
            left: Box::new(None),
            right: Box::new(None),
            frequency,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn binary_heap_tinkering() {
        let mut priority = BinaryHeap::new();

        priority.push(Root::new(30));
        priority.push(Root::new(10));
        priority.push(Root::new(20));

        assert_eq!(priority.pop().unwrap().frequency, 10);
        assert_eq!(priority.pop().unwrap().frequency, 20);
    }
}
