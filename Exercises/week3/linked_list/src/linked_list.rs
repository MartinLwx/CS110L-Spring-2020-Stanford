use std::fmt;
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node { value, next }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {
            head: None,
            size: 0,
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }

    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}

// add a trait bound
impl<T: fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    // I modify this format!() to get a better output
                    if result.len() == 0 {
                        result = format!("{}", node.value);
                    } else {
                        result = format!("{} {}", result, node.value);
                    }
                    current = &node.next;
                }
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

// Note: we should implement Clone trait on both of Node<T> and LinkedList<T>
// add trait bound
impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            next: self.next.clone(),
        }
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        LinkedList {
            head: self.head.clone(),
            size: self.size,
        }
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            false
        } else {
            let mut current1 = &self.head;
            let mut current2 = &other.head;
            loop {
                match (current1, current2) {
                    (None, _) => break,
                    (_, None) => break,
                    (Some(node1), Some(node2)) => {
                        if node1.value != node2.value {
                            return false;
                        }
                        current1 = &node1.next;
                        current2 = &node2.next;
                    }
                }
            }
            true
        }
    }

    fn ne(&self, other: &Self) -> bool {
        if self.size != other.size {
            true
        } else {
            let mut current1 = &self.head;
            let mut current2 = &other.head;
            loop {
                match (current1, current2) {
                    (None, _) => break,
                    (_, None) => break,
                    (Some(node1), Some(node2)) => {
                        if node1.value != node2.value {
                            return true;
                        }
                        current1 = &node1.next;
                        current2 = &node2.next;
                    }
                }
            }
            false
        }
    }
}

pub trait ComputeNorm {
    fn compute_norm(&self) -> f64 {
        // just the default value :)
        0.0
    }
}

// Note: now we implement the trait for a specific type rather than generic type T
impl ComputeNorm for LinkedList<f64> {
    fn compute_norm(&self) -> f64 {
        let mut sum: f64 = 0.0;
        if self.size > 0 {
            let mut current = &self.head;
            loop {
                match current {
                    None => break,
                    Some(node) => {
                        sum += node.value * node.value;
                        current = &node.next;
                    }
                }
            }
        }
        sum.sqrt()
    }
}
