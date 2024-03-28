#![allow(unused)]

use core::fmt;
use std::collections::LinkedList;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Code {
    data: Numeric,
    len: u8,
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Numeric {
    Usize(usize),
    U32(u32),
    U16(u16),
    U8(u8),
}

impl From<Numeric> for usize {
    fn from(value: Numeric) -> Self {
        match value {
            Numeric::Usize(v) => v,
            Numeric::U32(v) => v as usize,
            Numeric::U16(v) => v as usize,
            Numeric::U8(v) => v as usize,
        }
    }
}

impl Code {
    pub fn new(data: Numeric, len: u8) -> Self {
        Self { data, len }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BitIO {
    data: LinkedList<u8>,
    len: usize,
    read: u8,
}

impl BitIO {
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
            if usize::from(code.data) & (1 << i) != 0 {
                self.write_bit_back(true);
            } else {
                self.write_bit_back(false);
            }
        }
    }

    pub fn write_byte(&mut self, data: u8) {
        for i in 0..8 {
            if data & (1 << i) != 0 {
                self.write_bit_back(true);
            } else {
                self.write_bit_back(false);
            }
        }
    }
    pub fn write_u32_align_little_endian(&mut self, data: u32) {
        self.write_byte_align((data & 0xff) as u8);
        self.write_byte_align((data & 0xff00) as u8);
        self.write_byte_align((data & 0xff0000) as u8);
        self.write_byte_align((data & 0xff000000) as u8);
    }
    pub fn write_byte_align(&mut self, data: u8) {
        self.data.push_back(data);
    }

    pub fn write_code_rev(&mut self, code: &Code) {
        // NOTE: we have to reverse the code before writing
        for i in (0..code.len).rev() {
            if usize::from(code.data) & (1 << i) != 0 {
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

impl fmt::Debug for BitIO {
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
        let s: String = format!("{:032b}", usize::from(self.data))
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
        let mut handler = BitIO::new(LinkedList::new());
        handler.write_code(&Code {
            data: Numeric::Usize(97),
            len: 8,
        }); //
        handler.write_code(&Code {
            data: Numeric::Usize(229),
            len: 8,
        }); // 10100111
        handler.write_code(&Code {
            data: Numeric::Usize(1),
            len: 1,
        }); // 10100111
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
        let mut handler = BitIO {
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
        let mut handler = BitIO::new(LinkedList::new());
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
