#![allow(unused)]

use std::{
    collections::{BinaryHeap, HashMap, LinkedList},
    fs::File,
    io::Write,
    ptr,
};

use serde::{de, Serialize};

use crate::{
    bit_writer::{BitHandler, Code},
    graph_viz::GraphViz,
};
struct HuffNode {
    left: Node,
    right: Node,
    parent: Node,
    symbol: Option<char>,
    weight: usize,
    number: usize,
}

struct Node(*mut HuffNode);

struct Codec {
    block: HashMap<usize, BinaryHeap<Node>>,
    symbol_map: HashMap<char, Node>,
    nyt: Node,
    root: Node,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number().cmp(&other.number())
    }
}

impl GraphViz for Node {
    fn node_name(&self) -> String {
        format!("{}", self.number())
    }
    fn node_attribute(&self) -> String {
        let mut res = "".to_string();
        res.push_str(&self.node_name());
        res.push('[');
        res.push_str("label=\"");
        if self.is_leaf() {
            if self.is_nyt() {
                res.push_str(&format!("{} | NYT", self.weight()));
            } else {
                res.push_str(&format!("{} | {}", self.weight(), self.symbol().unwrap()));
            }
        } else {
            res.push_str(&format!("{}", self.weight()));
        }
        res.push_str("\",");
        res.push_str("shape=\"");
        if self.is_leaf() {
            res.push_str("record");
        } else {
            res.push_str("circle");
        }
        res.push_str("\",");
        res.push_str("xlabel=\"");
        res.push_str(&format!("{}", self.number()));
        res.push_str("\",");
        res.push(']');
        res
    }

    fn edge(&self) -> String {
        let mut res = "".to_string();
        for node in self.child() {
            res.push_str(&format!("{} -- {}\n", self.node_name(), node.node_name()));
        }

        res
    }

    fn child(&self) -> Vec<Box<impl GraphViz>> {
        let mut res = vec![];
        if !self.left().is_null() {
            res.push(Box::new(self.left()))
        }
        if !self.right().is_null() {
            res.push(Box::new(self.right()))
        }

        res
    }
}

impl Node {
    pub fn new(number: usize) -> Self {
        Self(Box::into_raw(Box::new(HuffNode {
            left: Self::null(),
            right: Self::null(),
            parent: Self::null(),
            symbol: None,
            weight: 0,
            number,
        })))
    }
    pub fn null() -> Self {
        Self(ptr::null_mut())
    }
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
    pub fn parent(&self) -> Self {
        if self.is_null() {
            return Self::null();
        }
        unsafe { (*self.0).parent.clone() }
    }
    pub fn right(&self) -> Self {
        if self.is_null() {
            return Self::null();
        }
        unsafe { (*self.0).right.clone() }
    }
    pub fn left(&self) -> Self {
        if self.is_null() {
            return Self::null();
        }
        unsafe { (*self.0).left.clone() }
    }
    pub fn set_parent(&mut self, node: Node) {
        if self.is_null() {
            return;
        }
        unsafe { (*self.0).parent = node }
    }
    pub fn set_right(&mut self, node: Node) {
        if self.is_null() {
            return;
        }
        unsafe { (*self.0).right = node }
    }
    pub fn set_left(&mut self, node: Node) {
        if self.is_null() {
            return;
        }
        unsafe { (*self.0).left = node }
    }
    pub fn set_weight(&mut self, weight: usize) {
        unsafe { (*self.0).weight = weight }
    }
    pub fn incr_weight(&mut self) {
        unsafe { (*self.0).weight += 1 }
    }
    pub fn weight(&self) -> usize {
        unsafe { (*self.0).weight }
    }
    pub fn set_number(&mut self, number: usize) {
        unsafe { (*self.0).number = number }
    }
    pub fn number(&self) -> usize {
        unsafe { (*self.0).number }
    }
    pub fn set_symbol(&mut self, symbol: char) {
        unsafe { (*self.0).symbol = Some(symbol) }
    }
    pub fn symbol(&self) -> Option<char> {
        unsafe { (*self.0).symbol }
    }
    pub fn is_right_child(&self) -> bool {
        if self.is_null() {
            return false;
        }
        self.parent().right() == *self
    }
    pub fn is_left_child(&self) -> bool {
        if self.is_null() {
            return false;
        }
        self.parent().left() == *self
    }

    pub fn is_leaf(&self) -> bool {
        self.symbol().is_some() || self.is_nyt()
    }
    pub fn is_nyt(&self) -> bool {
        self.weight() == 0
    }

    pub fn exchange(&mut self, other: Node) {
        let mut other_parent = other.parent();
        let mut self_parent = self.parent();
        if other.is_left_child() {
            other_parent.set_left(self.clone())
        } else {
            other_parent.set_right(self.clone());
        }
        if self.is_left_child() {
            self_parent.set_left(other.clone());
        } else {
            self_parent.set_right(other.clone());
        }

        unsafe {
            ((*self.0).parent, (*other.0).parent) = (other_parent, self_parent);
        }
    }
}

impl Codec {
    pub fn new() -> Self {
        let nyt = Node::new(u32::MAX as usize);
        Self {
            block: HashMap::new(),
            symbol_map: HashMap::new(),
            nyt: nyt.clone(),
            root: nyt,
        }
    }

    pub fn write_symbol(&mut self, symbol: char, handler: &mut BitHandler) {
        if self.symbol_map.contains_key(&symbol) {
            let node = self.symbol_map.get(&symbol).unwrap().clone();
            let code = Self::code_from_node(node.clone());

            #[cfg(test)]
            println!("write symbol: {}, code: {code:?}", node.symbol().unwrap());
            handler.write_code(&code);
            self.update_node(node);
        } else {
            let code = Self::code_from_node(self.nyt.clone());
            #[cfg(test)]
            println!("write nyt, code: {code:?}");
            handler.write_code_rev(&code);

            let node = self.new_node(symbol);
            self.symbol_map.insert(symbol, node.clone());

            let code = self.code_from_symbol(symbol);

            #[cfg(test)]
            println!("write symbol: {}, code: {code:?}", symbol);
            handler.write_code(&code);
            self.update_node(node);
        }
    }

    pub fn encode(&mut self, input: &String) -> BitHandler {
        let mut handler = BitHandler::new(LinkedList::new());
        #[cfg(test)]
        let mut file = File::create("tree.dot").unwrap();
        #[cfg(test)]
        let mut buffer: Vec<u8> = vec![];
        for c in input.chars() {
            self.write_symbol(c, &mut handler);
            #[cfg(test)]
            {
                let script = self.root.draw_to_string();
                writeln!(buffer, "{script}");
            }
        }
        #[cfg(test)]
        {
            file.write(&buffer);
        }
        handler
    }

    pub fn decode(&mut self, handler: &mut BitHandler) -> String {
        let mut res = "".to_string();
        let mut node = self.root.clone();
        while !handler.is_empty() {
            if !node.is_leaf() {
                match handler.read_bit_front().unwrap() {
                    true => node = node.right(),
                    false => node = node.left(),
                }
            } else {
                if node.is_nyt() {
                    let mut num = 0;
                    for i in 0..32 {
                        if handler.read_bit_front().unwrap() {
                            num |= (1 << i);
                        }
                    }
                    let symbol = char::from_u32(num).unwrap();
                    res.push(symbol);
                    node = self.new_node(symbol);
                } else {
                    let symbol = node.symbol().unwrap();
                    res.push(symbol);
                }
                println!("res: {res}");
                self.update_node(node.clone());
                node = self.root.clone();
            }
        }
        if node.is_leaf() && !node.is_nyt() {
            res.push(node.symbol().unwrap());
        }
        res
    }

    fn code_from_node(node: Node) -> Code {
        let mut path = 0usize;
        let mut depth = 0;

        let mut a = node.clone();
        while !a.parent().is_null() {
            if a.is_right_child() {
                path |= (1 << depth);
            }
            depth += 1;
            a = a.parent();
        }

        let code = Code::new(path, depth);

        code
    }

    fn code_from_symbol(&self, symbol: char) -> Code {
        // TODO: we should use utf8 encoding, instead of u32
        // char to bytes: https://github.com/rust-lang/rust/blob/9cc0b2247509d61d6a246a5c5ad67f84b9a2d8b6/src/libcore/char.rs#L220
        Code::new(symbol as u32 as usize, 32u8)
    }

    fn new_node(&mut self, symbol: char) -> Node {
        let cur_number = self.nyt.number();
        let mut p = Node::new(cur_number);
        let mut r = Node::new(cur_number - 1);
        self.nyt.set_number(cur_number - 2);
        p.set_parent(self.nyt.parent());
        if !self.nyt.parent().is_null() {
            self.nyt.parent().set_left(p.clone());
        }
        self.nyt.set_parent(p.clone());
        p.set_left(self.nyt.clone());
        p.set_right(r.clone());
        r.set_parent(p.clone());
        r.set_symbol(symbol);
        assert!(r.is_right_child());
        assert!(self.nyt.is_left_child());
        self.push_node_to_block(r.clone(), 0);
        self.push_node_to_block(p.clone(), 0);
        r.clone()
    }

    fn update_node(&mut self, mut node: Node) {
        if node.is_null() {
            return;
        }
        if let Some(other) = self.is_max_in_block(node.clone()) {
            node.exchange(other);
        }

        self.update_node_in_block(node.clone());
        node.incr_weight();

        if node.parent().is_null() {
            self.root = node.clone();
        } else {
            self.update_node(node.parent());
        }
    }

    fn push_node_to_block(&mut self, node: Node, weight: usize) {
        if !self.block.contains_key(&weight) {
            self.block.insert(weight, BinaryHeap::new());
        }
        if let Some(arr) = self.block.get_mut(&weight) {
            arr.push(node);
        }
    }

    fn is_max_in_block(&mut self, node: Node) -> Option<Node> {
        let weight = node.weight();
        let p = node.parent();
        let heap = self.block.get_mut(&weight).unwrap();
        let max_node = heap.pop().unwrap();
        if node == max_node {
            heap.push(max_node);
            return None;
        } else if p == max_node {
            let second_max_node = heap.pop().unwrap();
            heap.push(max_node);
            if second_max_node == node {
                heap.push(second_max_node);
                return None;
            } else {
                heap.push(second_max_node.clone());
                return Some(second_max_node);
            }
        } else {
            heap.push(max_node.clone());
            return Some(max_node.clone());
        }
    }

    fn update_node_in_block(&mut self, node: Node) {
        let weight = node.weight();

        // remove node in previous block
        let mut new_heap = BinaryHeap::new();
        if let Some(arr) = self.block.get(&weight) {
            arr.iter().for_each(|n| {
                if *n != node {
                    new_heap.push(n.clone());
                }
            });
        } else {
            panic!("cannot find block weighted {}", weight);
        }
        self.block.insert(weight, new_heap);
        self.push_node_to_block(node, weight + 1);
    }
}

impl Drop for Codec {
    fn drop(&mut self) {
        for (k, v) in &self.block {
            for node in v {
                unsafe {
                    let _ = Box::from_raw(node.0);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node() {
        let mut node = Node::new(0);

        let mut arr = vec![node.clone()];

        node.set_number(1);

        assert!(arr[0].number() == 1);

        arr.remove(0);
        assert!(node.number() == 1);
    }

    #[test]
    fn test_adaptive_huffman() {
        let mut codec = Codec::new();
        let input = "aardvss".to_string();
        let mut handler = codec.encode(&input);
        println!("{handler:?}");
        let mut codec = Codec::new();
        let res = codec.decode(&mut handler);
        assert_eq!(input, res);
    }
}
