#![allow(unused)]
use std::collections::{BinaryHeap, HashMap};

use crate::bit_writer::{BitHandler, Code};

struct NodeRaw {
    left: Node,
    right: Node,
    symbol: Option<char>,
    freq: usize,
}

type Node = Option<Box<NodeRaw>>;

impl PartialOrd for NodeRaw {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for NodeRaw {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}
impl Eq for NodeRaw {}
impl Ord for NodeRaw {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq.cmp(&other.freq).reverse()
    }
}

pub struct Codec {
    root: Node,
    symbol_map: HashMap<char, Code>,
}

impl Codec {
    pub fn new() -> Self {
        Self {
            root: None,
            symbol_map: HashMap::new(),
        }
    }

    pub fn encode(&mut self, input: String) -> BitHandler {
        // get frequency of each char
        let mut freq = HashMap::new();
        input.chars().for_each(|c| {
            freq.insert(c, freq.get(&c).unwrap_or(&0) + 1);
        });
        let freq: Vec<(char, usize)> = freq.iter().map(|(&k, &v)| (k, v)).collect();
        let symbol_size = freq.len();

        // make huffman tree
        let mut heap = BinaryHeap::new();
        for (symbol, f) in freq {
            heap.push(Self::new_leaf_node(symbol, f));
        }
        for _ in 1..symbol_size {
            let left = heap.pop().unwrap().expect("left node");
            let right = heap.pop().unwrap().expect("right node");
            let freq = left.freq + right.freq;
            heap.push(Self::new_internal_node(Some(left), Some(right), freq))
        }
        self.root = heap.pop().unwrap();
        Self::pollute_symbol_map(&self.root, &mut self.symbol_map, 0, 0);

        // write to res
        let mut writer = BitHandler::new(vec![]);
        input.chars().for_each(|c| {
            writer.write_code(self.symbol_map.get(&c).expect("get code from symbol_map"));
        });

        // println!("{writer:?}");
        // println!("{:?}", self.symbol_map);

        writer
    }

    pub fn decode(&self, input: &mut BitHandler) -> String {
        assert!(self.root.is_some() && self.symbol_map.len() != 0);
        let mut a = self.root.as_ref().unwrap().as_ref();
        let mut res = "".to_string();
        while !input.is_empty() {
            let next = input.read_bit().unwrap();
            a = if next {
                a.right.as_ref().unwrap()
            } else {
                a.left.as_ref().unwrap()
            };
            if let Some(symbol) = a.symbol {
                res.push(symbol);
                a = self.root.as_ref().unwrap().as_ref();
            }
        }
        res.chars().into_iter().rev().collect()
    }

    fn pollute_symbol_map(node: &Node, map: &mut HashMap<char, Code>, depth: u8, code: usize) {
        if let Some(node) = node {
            match node.symbol {
                Some(symbol) => {
                    map.insert(symbol, Code::new(code, depth));
                }
                None => {
                    Self::pollute_symbol_map(&node.left, map, depth + 1, code);
                    let code = code | (1 << depth);
                    Self::pollute_symbol_map(&node.right, map, depth + 1, code);
                }
            }
        }
    }

    fn new_leaf_node(symbol: char, freq: usize) -> Node {
        Some(Box::new(NodeRaw {
            left: None,
            right: None,
            symbol: Some(symbol),
            freq,
        }))
    }

    fn new_internal_node(left: Node, right: Node, freq: usize) -> Node {
        Some(Box::new(NodeRaw {
            left,
            right,
            symbol: None,
            freq,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let input = "hello world".to_string();
        let mut codec = Codec::new();
        let mut handler = codec.encode(input);
        let res = codec.decode(&mut handler);
        assert_eq!(res, "hello world");
    }
}
