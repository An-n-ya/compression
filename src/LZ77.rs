#![allow(unused)]
use std::collections::LinkedList;

use crate::bit_io::Reader;

pub struct Codec {
    window_size: usize,
    look_ahead_size: usize,
    min_match_size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Literal(u8),
    BackRef { len: usize, distance: usize },
}

impl Codec {
    pub fn new(window_size: usize, look_ahead_size: usize, min_match_size: usize) -> Self {
        assert!(window_size > 0 && look_ahead_size > 0);
        Self {
            window_size,
            look_ahead_size,
            min_match_size,
        }
    }

    pub fn encode(&self, input: &[u8]) -> Vec<Value> {
        let mut search_window: LinkedList<u8> = LinkedList::new();
        let mut look_ahead_window: LinkedList<u8> = LinkedList::new();
        let mut reader = Reader::new(input);
        for _ in 0..self.look_ahead_size {
            if let Some(val) = reader.read_u8() {
                look_ahead_window.push_back(val);
            } else {
                break;
            }
        }
        let mut res = vec![];
        while !reader.is_empty() || !look_ahead_window.is_empty() {
            let cur_val = *look_ahead_window.front().unwrap();
            let mut match_len = 0;
            if let Some((index, len)) = self.max_match(&search_window, &look_ahead_window) {
                let distance = search_window.len() - index;
                match_len = len;
                res.push(Value::BackRef { len, distance });
            } else {
                res.push(Value::Literal(cur_val));
            }

            for _ in 0..1.max(match_len) {
                if search_window.len() == self.window_size {
                    search_window.pop_front();
                }
                let cur_val = look_ahead_window.pop_front().unwrap();
                search_window.push_back(cur_val);
                if let Some(next_val) = reader.read_u8() {
                    look_ahead_window.push_back(next_val);
                }
            }
            // if match_len > 0 {
            //     println!("look_ahead_window: {look_ahead_window:?}");
            //     println!("search_window: {search_window:?}");
            // }
        }
        res
    }

    fn max_match(&self, list1: &LinkedList<u8>, list2: &LinkedList<u8>) -> Option<(usize, usize)> {
        let mut max_match_len = 0;
        let mut match_index = 0;
        for skip in 0..list1.len() {
            for (i, (n1, n2)) in list1.iter().skip(skip).zip(list2).enumerate() {
                if n1 != n2 {
                    if i >= max_match_len && i >= self.min_match_size {
                        max_match_len = i;
                        match_index = skip;
                    }
                    break;
                }
                if i == list1.len() - 1 && i < list2.len() - 1 {
                    // compare prefix and suffix
                    let mut self_max_match_len = 0;
                    for (ii, (n1, n2)) in list2.iter().skip(i + 1).zip(list2).enumerate() {
                        if n1 != n2 {
                            if ii >= self_max_match_len && ii + i >= self.min_match_size {
                                self_max_match_len = ii;
                            }
                            break;
                        }
                    }
                    max_match_len = i + 1 + self_max_match_len;
                    match_index = skip;
                    break;
                }
                if i == list2.len() - 1 {
                    if i + 1 >= self.min_match_size {
                        max_match_len = i + 1;
                        match_index = skip;
                    }
                }
            }
        }
        if max_match_len == 0 {
            None
        } else {
            Some((match_index, max_match_len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_basic() {
        let input = b"A SALAD; A SALSA";
        let expected = vec![
            Value::Literal(b'A'),
            Value::Literal(b' '),
            Value::Literal(b'S'),
            Value::Literal(b'A'),
            Value::Literal(b'L'),
            Value::Literal(b'A'),
            Value::Literal(b'D'),
            Value::Literal(b';'),
            Value::Literal(b' '),
            Value::BackRef {
                len: 5,
                distance: 9,
            },
            Value::BackRef {
                len: 2,
                distance: 3,
            },
        ];
        let codec = Codec::new(100, 100, 2);
        let res = codec.encode(input);
        println!("{res:?}");
        res.iter().zip(expected.iter()).for_each(|(v1, v2)| {
            assert_eq!(v1, v2);
        })
    }
}
