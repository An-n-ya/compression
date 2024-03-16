#![allow(unused)]

use core::fmt;
use std::collections::LinkedList;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Code {
    data: usize,
    len: u8,
}

impl Code {
    pub fn new(data: usize, len: u8) -> Self {
        Self { data, len }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BitHandler {
    data: LinkedList<u8>,
    len: usize,
    read: u8,
}

impl BitHandler {
    pub fn new(data: LinkedList<u8>) -> Self {
        let len = data.len() * 8;
        Self { data, len, read: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn read_bit_front(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.read = (self.read + 1) % 8;
        let top = self.data.pop_front().unwrap();
        let res = top & 1 != 0;
        if self.read % 8 != 0 {
            self.data.push_front(top >> 1);
        }
        Some(res)
    }

    pub fn read_bit_back(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let top = self.data.pop_back().unwrap();
        let offset = self.len % 8;
        let res = top & (1 << offset) != 0;
        if self.len % 8 != 0 {
            self.data.push_back(top);
        }
        Some(res)
    }

    pub fn write_bit_back(&mut self, bit: bool) {
        if self.len % 8 == 0 {
            self.data.push_back(0);
        }
        if bit {
            let offset = self.len % 8;
            let top = self.data.pop_back().unwrap();
            self.data.push_back(top | (1 << offset));
        }
        self.len += 1;
    }

    pub fn write_code(&mut self, code: &Code) {
        // NOTE: we have to reverse the code before writing
        for i in (0..code.len) {
            if code.data & (1 << i) != 0 {
                self.write_bit_back(true);
            } else {
                self.write_bit_back(false);
            }
        }
    }
    pub fn write_code_rev(&mut self, code: &Code) {
        // NOTE: we have to reverse the code before writing
        for i in (0..code.len).rev() {
            if code.data & (1 << i) != 0 {
                self.write_bit_back(true);
            } else {
                self.write_bit_back(false);
            }
        }
    }

    pub fn as_vec(self) -> LinkedList<u8> {
        self.data
    }
}

impl fmt::Debug for BitHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut len = self.len;
        for n in &self.data {
            let s: String = format!("{:08b}", n).chars().into_iter().rev().collect();
            if len > 8 {
                len -= 8;
                write!(f, "{s} ").unwrap()
            } else {
                let s = &s[0..len];
                write!(f, "{s} ").unwrap()
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = format!("{:032b}", self.data)
            .chars()
            .into_iter()
            .rev()
            .collect();
        let s = &s[0..self.len as usize];
        write!(f, "{s}({})", self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_front() {
        let mut handler = BitHandler::new(LinkedList::new());
        handler.write_code(&Code { data: 97, len: 8 }); //
        handler.write_code(&Code { data: 229, len: 8 }); // 10100111
        handler.write_code(&Code { data: 1, len: 1 }); // 10100111
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), false);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front().unwrap(), true);
        assert_eq!(handler.read_bit_front(), None);
    }
    #[test]
    fn test_read() {
        let mut handler = BitHandler {
            data: LinkedList::from_iter(vec![0, 5]),
            len: 11,
            read: 0,
        };
        assert_eq!(handler.read_bit_back().unwrap(), true);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), true);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back(), None);
    }

    #[test]
    fn test_write() {
        let mut handler = BitHandler::new(LinkedList::new());
        handler.write_bit_back(true);
        handler.write_bit_back(false);
        handler.write_bit_back(true);
        handler.write_bit_back(false);
        assert_eq!(handler.len, 4);
        assert_eq!(handler.data.front().unwrap(), &5);

        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), true);
        assert_eq!(handler.read_bit_back().unwrap(), false);
        assert_eq!(handler.read_bit_back().unwrap(), true);
        assert_eq!(handler.read_bit_back(), None);
    }
}
