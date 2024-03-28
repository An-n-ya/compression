#![allow(unused)]
use std::{
    collections::{BinaryHeap, HashMap, LinkedList},
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader, Read, Write},
    os::unix::fs::FileExt,
};

use serde::{Deserialize, Serialize};
use serde_cbor::{from_slice, ser::to_vec_packed};

use crate::{
    bit_io::{BitIO, Code},
    utils::freq_of_str,
};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

    pub fn encode(&mut self, input: &String) -> BitIO {
        let freq = freq_of_str(&input);
        // get frequency of each char
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
        let mut writer = BitIO::new(LinkedList::new());
        input.chars().for_each(|c| {
            writer.write_code_rev(self.symbol_map.get(&c).expect("get code from symbol_map"));
        });

        // println!("{writer:?}");
        // println!("{:?}", self.symbol_map);

        writer
    }

    pub fn decode(&self, input: &mut BitIO) -> String {
        assert!(self.root.is_some());
        let mut a = self.root.as_ref().unwrap().as_ref();
        let mut res = "".to_string();
        while !input.is_empty() {
            let next = input.read_bit_back().unwrap();
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

    pub fn persist_to_file(&self, output: &BitIO) {
        let mut file = File::create("compression.huff").unwrap();
        let mut header = to_vec_packed(&self.root).unwrap();
        let mut header_len: Vec<u8> = format!("{}", header.len())
            .chars()
            .map(|c| c as u8)
            .collect();
        header_len.push(b'\0');
        header_len.extend(header);
        let data = to_vec_packed(output).unwrap();
        header_len.extend(data);
        file.write_all(&header_len);
    }

    pub fn decode_from_file(file_name: &str) -> String {
        let mut file = File::open(file_name).unwrap();
        let mut file = BufReader::new(file);
        let mut header_len: Vec<u8> = vec![];
        file.read_until(b'\0', &mut header_len).unwrap();
        header_len.pop();
        let header_len: String = header_len.iter().map(|n| *n as char).collect();
        let header_len = header_len.parse::<usize>().unwrap();

        let mut buf = vec![0; header_len];
        file.read_exact(&mut buf).unwrap();
        let root: Node = from_slice(&buf).unwrap();
        let codec = Codec {
            root,
            symbol_map: HashMap::new(),
        };
        let mut data: Vec<u8> = vec![];
        file.read_to_end(&mut data);
        let mut data: BitIO = from_slice(&data).unwrap();
        let res = codec.decode(&mut data);
        res
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
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use super::*;

    #[test]
    fn test_encode() {
        let input = "hello world".to_string();
        let mut codec = Codec::new();
        let mut handler = codec.encode(&input);
        let res = codec.decode(&mut handler);
        assert_eq!(res, "hello world");
    }

    // #[test]
    fn test_hlm() {
        let mut file = File::open("hlm.txt").unwrap();
        let mut input = "".to_string();
        file.read_to_string(&mut input);

        let mut codec = Codec::new();
        let mut handler = codec.encode(&input);

        codec.persist_to_file(&handler);

        let res = Codec::decode_from_file("compression.huff");
        assert_eq!(res, input);
    }
}
