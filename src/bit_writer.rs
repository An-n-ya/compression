#![allow(unused)]

use core::fmt;
pub struct Code {
    data: usize,
    len: u8,
}

impl Code {
    pub fn new(data: usize, len: u8) -> Self {
        Self { data, len }
    }
}

pub struct BitHandler {
    data: Vec<u8>,
    len: usize,
}

impl BitHandler {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, len: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let top = self.data.pop().unwrap();
        let offset = self.len % 8;
        let res = top & (1 << offset) != 0;
        if self.len % 8 != 0 {
            self.data.push(top);
        }
        Some(res)
    }

    pub fn write_bit(&mut self, bit: bool) {
        if self.len % 8 == 0 {
            self.data.push(0);
        }
        if bit {
            let offset = self.len % 8;
            let top = self.data.pop().unwrap();
            self.data.push(top | (1 << offset));
        }
        self.len += 1;
    }

    pub fn write_code(&mut self, code: &Code) {
        // NOTE: we have to reverse the code before writing
        for i in (0..code.len).rev() {
            if code.data & (1 << i) != 0 {
                self.write_bit(true);
            } else {
                self.write_bit(false);
            }
        }
    }

    pub fn as_vec(self) -> Vec<u8> {
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
        let s: String = format!("{:08b}", self.data)
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
    fn test_read() {
        let mut handler = BitHandler {
            data: vec![0, 5],
            len: 11,
        };
        assert_eq!(handler.read_bit().unwrap(), true);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), true);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit(), None);
    }

    #[test]
    fn test_write() {
        let mut handler = BitHandler::new(vec![]);
        handler.write_bit(true);
        handler.write_bit(false);
        handler.write_bit(true);
        handler.write_bit(false);
        assert_eq!(handler.len, 4);
        assert_eq!(handler.data[0], 5);

        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), true);
        assert_eq!(handler.read_bit().unwrap(), false);
        assert_eq!(handler.read_bit().unwrap(), true);
        assert_eq!(handler.read_bit(), None);
    }
}
