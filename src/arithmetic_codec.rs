#![allow(unused)]

use std::{
    collections::{HashMap, LinkedList},
    io::BufReader,
    ops::Add,
};

use crate::{bit_writer::BitHandler, utils::freq_of_str};
pub struct FloatCodec {
    symbol_size: usize,
    freq: Vec<(char, f64)>,
}

impl FloatCodec {
    pub fn new_with_str(symbol_size: usize, input: &str) -> Self {
        let freq = freq_of_str(&input);
        let freq: Vec<(char, f64)> = freq
            .iter()
            .map(|(c, cnt)| (*c, (*cnt as f64 / symbol_size as f64)))
            .collect();
        Self { symbol_size, freq }
    }
    pub fn new_with_freq(symbol_size: usize, freq: Vec<(char, f64)>) -> Self {
        Self { symbol_size, freq }
    }

    pub fn encode(&self, input: &str) -> BitHandler {
        let (mut l, mut u) = (0f64, 1f64);
        let fx = Fx::new(&self.freq);
        let mut cur_index = 0;
        let input: Vec<char> = input.chars().collect();
        let mut handler = BitHandler::new(LinkedList::new());
        let mut e3_cnt = 0;
        while cur_index < input.len() {
            let c = input[cur_index];
            let fx_l = fx.get_prev_val(c);
            let fx_r = fx.get_val(c);
            (l, u) = (l + (u - l) * fx_l, l + (u - l) * fx_r);
            assert!(l < u);
            if l < 0.5 && u > 0.5 {
                while l > 0.25 && u < 0.75 {
                    (l, u) = (2.0 * (l - 0.25), 2.0 * (u - 0.25));
                    e3_cnt += 1;
                }
                println!(
                    "c: {c} fx_l:{fx_l:.4} fx_r:{fx_r:.4}, l:{l:.4}, u:{u:.4}, e3_cnt:{e3_cnt}"
                );
                cur_index += 1;
                continue;
            }
            println!("c: {c} fx_l:{fx_l:.4} fx_r:{fx_r:.4}, l:{l:.4}, u:{u:.4}");
            while l > 0.5 || u < 0.5 {
                if l > 0.5 {
                    handler.write_bit_back(true);
                    (l, u) = (2f64 * (l - 0.5), 2f64 * (u - 0.5));
                    while e3_cnt > 0 {
                        handler.write_bit_back(false);
                        e3_cnt -= 1;
                    }
                } else if u < 0.5 {
                    handler.write_bit_back(false);
                    (l, u) = (2f64 * l, 2f64 * u);
                    while e3_cnt > 0 {
                        handler.write_bit_back(true);
                        e3_cnt -= 1;
                    }
                }
            }
            cur_index += 1;
        }
        handler.write_bit_back(true);
        for _ in 0..self.symbol_size {
            handler.write_bit_back(false);
        }

        handler
    }

    fn decode(&self, handler: BitHandler) -> String {
        let mut reader = FloatReader::new(self.symbol_size, handler);
        let (mut l, mut u) = (0.0, 1.0);
        let fx = Fx::new(&self.freq);
        let mut cur_label = reader.cur_val();
        let mut res = "".to_string();

        while !reader.is_empty() {
            let proportion = (cur_label - l) / (u - l);
            println!("cur_label:{cur_label}, l:{l:.4}, u:{u:.4} proportion: {proportion:.4}");
            let c = fx.find_char(proportion);
            println!("find char {c}");
            res.push(c);
            if cur_label == 0.5 {
                return res;
            }
            let (fx_l, fx_r) = (fx.get_prev_val(c), fx.get_val(c));
            (l, u) = (l + (u - l) * fx_l, l + (u - l) * fx_r);
            println!("l:{l:.4}, u:{u:.4}, fx_l:{fx_l:.4}, fx_r:{fx_r:.4}");
            assert!(l < u);
            if l < 0.5 && u > 0.5 {
                continue;
            }
            while l > 0.5 || u < 0.5 {
                cur_label = reader.next();
                if l > 0.5 {
                    (l, u) = (2f64 * (l - 0.5), 2f64 * (u - 0.5));
                } else if u < 0.5 {
                    (l, u) = (2f64 * l, 2f64 * u);
                }
            }
        }

        res
    }
}

struct FloatReader {
    symbol_size: usize,
    cur_val: f64,
    handler: BitHandler,
    base: f64,
}

impl FloatReader {
    pub fn new(symbol_size: usize, mut handler: BitHandler) -> Self {
        let mut cur_val = 0f64;
        let mut base = 1.0;
        for _ in 0..symbol_size {
            base = base / 2.0;
            if handler.read_bit_front().unwrap() {
                cur_val += base
            }
        }
        Self {
            symbol_size,
            cur_val,
            handler,
            base,
        }
    }
    pub fn cur_val(&self) -> f64 {
        self.cur_val
    }
    pub fn next(&mut self) -> f64 {
        self.cur_val *= 2.0;
        if self.cur_val > 1.0 {
            self.cur_val -= 1.0;
        }
        if self.handler.read_bit_front().unwrap() {
            self.cur_val += self.base;
        }
        self.cur_val
    }
    pub fn is_empty(&self) -> bool {
        self.handler.is_empty()
    }
}

struct Fx {
    symbol_to_index: HashMap<char, usize>,
    index_to_symbol: HashMap<usize, char>,
    freq: HashMap<usize, f64>,
}

impl Fx {
    pub fn new(freq: &Vec<(char, f64)>) -> Fx {
        let mut cur_freq = 0f64;
        let mut fx_freq = HashMap::new();
        let mut symbol_to_index = HashMap::new();
        let mut index_to_symbol = HashMap::new();

        for (i, (c, f)) in freq.iter().enumerate() {
            symbol_to_index.insert(*c, i + 1);
            index_to_symbol.insert(i + 1, *c);

            cur_freq += *f;
            fx_freq.insert(i + 1, cur_freq);
        }
        fx_freq.insert(0, 0f64);

        Fx {
            symbol_to_index,
            index_to_symbol,
            freq: fx_freq,
        }
    }
    pub fn get_val(&self, c: char) -> f64 {
        *self
            .freq
            .get(self.symbol_to_index.get(&c).unwrap())
            .unwrap()
    }
    pub fn get_prev_val(&self, c: char) -> f64 {
        *self
            .freq
            .get(&(self.symbol_to_index.get(&c).unwrap() - 1))
            .unwrap()
    }
    pub fn find_char(&self, f: f64) -> char {
        assert!(f <= 1.0);
        for i in 1..self.freq.len() {
            let cur_f = *self.freq.get(&i).unwrap();
            // FIXME: `>=` or `>`?
            if cur_f >= f {
                return *self.index_to_symbol.get(&i).unwrap();
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let freq = vec![('a', 0.8f64), ('b', 0.02f64), ('c', 0.18f64)];
        let mut codec = FloatCodec::new_with_freq(6, freq);
        let input = "acba";
        let handler = codec.encode(input);
        println!("{handler:?}");
        let res = codec.decode(handler);
        println!("{res}");
        assert_eq!(res, input);
        panic!()
    }
}
