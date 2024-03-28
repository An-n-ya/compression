#![allow(unused)]

use std::{
    collections::{HashMap, LinkedList},
    io::BufReader,
    ops::Add,
};

use crate::{bit_io::BitIO, utils::freq_of_str};
pub struct Codec {
    symbol_size: usize,
    freq: Vec<(char, Val)>,
    total_cnt: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Val {
    Float(f64),
    Int(usize),
}

impl Codec {
    pub fn new_with_str(symbol_size: usize, input: &str, total_cnt: Option<usize>) -> Self {
        let freq = freq_of_str(&input);
        let freq: Vec<(char, Val)> = freq
            .iter()
            .map(|(c, cnt)| (*c, Val::Float(*cnt as f64 / symbol_size as f64)))
            .collect();
        Self {
            symbol_size,
            freq,
            total_cnt,
        }
    }
    pub fn new_with_int_freq(
        symbol_size: usize,
        freq: Vec<(char, usize)>,
        total_cnt: Option<usize>,
    ) -> Self {
        let freq: Vec<(char, Val)> = freq.iter().map(|(c, cnt)| (*c, Val::Int(*cnt))).collect();
        Self {
            symbol_size,
            freq,
            total_cnt,
        }
    }
    pub fn new_with_float_freq(
        symbol_size: usize,
        freq: Vec<(char, f64)>,
        total_cnt: Option<usize>,
    ) -> Self {
        let freq: Vec<(char, Val)> = freq.iter().map(|(c, cnt)| (*c, Val::Float(*cnt))).collect();
        Self {
            symbol_size,
            freq,
            total_cnt,
        }
    }

    pub fn encode(&self, input: &str) -> BitIO {
        let (mut l, mut u) = if self.total_cnt.is_some() {
            (Val::Int(0), Val::Int((1 << self.symbol_size) - 1))
        } else {
            (Val::Float(0.0), Val::Float(1.0))
        };
        let fx = Fx::new(&self.freq);
        let mut cur_index = 0;
        let input: Vec<char> = input.chars().collect();
        let mut handler = BitIO::new(LinkedList::new());
        let mut e3_cnt = 0;
        while cur_index < input.len() {
            let c = input[cur_index];
            let fx_l = fx.get_prev_val(c);
            let fx_r = fx.get_val(c);
            (l, u) = self.update_l_and_u(l, u, fx_l, fx_r);
            assert!(l < u);
            println!("c: {c} fx_l:{fx_l:?} fx_r:{fx_r:?}, l:{l:?}, u:{u:?} e3_cnt:{e3_cnt}");
            while self.in_e1(l, u) || self.in_e2(l, u) || self.in_e3(l, u) {
                if self.in_e2(l, u) {
                    handler.write_bit_back_to_lsb(true);
                    (l, u) = self.handle_e2(l, u);
                    while e3_cnt > 0 {
                        handler.write_bit_back_to_lsb(false);
                        e3_cnt -= 1;
                    }
                } else if self.in_e1(l, u) {
                    handler.write_bit_back_to_lsb(false);
                    (l, u) = self.handle_e1(l, u);
                    while e3_cnt > 0 {
                        handler.write_bit_back_to_lsb(true);
                        e3_cnt -= 1;
                    }
                } else if self.in_e3(l, u) {
                    (l, u) = self.handle_e3(l, u);
                    e3_cnt += 1;
                } else {
                    unreachable!()
                }
            }
            println!("c: {c} fx_l:{fx_l:?} fx_r:{fx_r:?}, l:{l:?}, u:{u:?} e3_cnt:{e3_cnt}");
            println!("{handler:?}");
            cur_index += 1;
        }
        self.write_end(&mut handler, u);

        handler
    }
    fn decode(&self, handler: BitIO) -> String {
        let mut reader = NumReader::new(self.symbol_size, handler, self.total_cnt.is_some());
        let (mut l, mut u) = if self.total_cnt.is_some() {
            (Val::Int(0), Val::Int((1 << self.symbol_size) - 1))
        } else {
            (Val::Float(0.0), Val::Float(1.0))
        };
        let fx = Fx::new(&self.freq);
        let mut cur_label = reader.cur_val();
        let mut res = "".to_string();

        while !reader.is_empty() {
            println!(
                "cur_label:{cur_label:?}({}), l:{l:?}, u:{u:?}",
                reader.cur_bit()
            );
            let c = fx.find_char(cur_label, l, u, self.total_cnt);
            println!("find char {c}");
            res.push(c);
            if cur_label == u {
                return res;
            }
            let (fx_l, fx_r) = (fx.get_prev_val(c), fx.get_val(c));
            (l, u) = self.update_l_and_u(l, u, fx_l, fx_r);
            assert!(l < u);
            while self.in_e1(l, u) || self.in_e2(l, u) || self.in_e3(l, u) {
                cur_label = reader.next();
                if self.in_e2(l, u) {
                    (l, u) = self.handle_e2(l, u);
                } else if self.in_e1(l, u) {
                    (l, u) = self.handle_e1(l, u);
                } else if self.in_e3(l, u) {
                    (l, u) = self.handle_e3(l, u);
                    cur_label = match cur_label {
                        Val::Float(v) => {
                            let mut v = v;
                            if v > 0.5 {
                                v -= 0.5;
                            } else {
                                v += 0.5;
                            }
                            Val::Float(v)
                        }
                        Val::Int(v) => {
                            let mut v = v;
                            v ^= 1 << (self.symbol_size - 1);
                            Val::Int(v)
                        }
                    };
                } else {
                    unreachable!()
                }
            }
            // println!("l:{l:.4}, u:{u:.4}, fx_l:{fx_l:.4}, fx_r:{fx_r:.4}");
        }

        res
    }

    fn write_end(&self, handler: &mut BitIO, end: Val) {
        match end {
            Val::Float(val) => {
                let mut val = val;
                let mut base = 0.5;
                for _ in 0..self.symbol_size {
                    if val >= base {
                        handler.write_bit_back_to_lsb(true);
                        val -= base;
                    } else {
                        handler.write_bit_back_to_lsb(false);
                    }
                    base /= 2.0;
                }
            }
            Val::Int(val) => {
                for i in (0..self.symbol_size).rev() {
                    if val & (1 << i) != 0 {
                        handler.write_bit_back_to_lsb(true);
                    } else {
                        handler.write_bit_back_to_lsb(false);
                    }
                }
            }
        };
    }

    fn handle_e1(&self, l: Val, u: Val) -> (Val, Val) {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return (Val::Float(l * 2.0), Val::Float(u * 2.0));
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let max = 1 << self.symbol_size;
                return (Val::Int((l << 1) % max), Val::Int((u << 1) % max + 1));
            }
        };
        unreachable!()
    }
    fn in_e1(&self, l: Val, u: Val) -> bool {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return l < 0.5 && u < 0.5;
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let msb = 1 << (self.symbol_size - 1);
                return l & msb == 0 && u & msb == 0;
            }
        };
        unreachable!()
    }

    fn handle_e2(&self, l: Val, u: Val) -> (Val, Val) {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return (Val::Float((l - 0.5) * 2.0), Val::Float((u - 0.5) * 2.0));
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let max = 1 << self.symbol_size;
                return (Val::Int((l << 1) % max), Val::Int((u << 1) % max + 1));
            }
        };
        unreachable!()
    }
    fn in_e2(&self, l: Val, u: Val) -> bool {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return l > 0.5 && u > 0.5;
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let msb = 1 << (self.symbol_size - 1);
                return l & msb != 0 && u & msb != 0;
            }
        };
        unreachable!()
    }
    fn handle_e3(&self, l: Val, u: Val) -> (Val, Val) {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return (Val::Float((l - 0.25) * 2.0), Val::Float((u - 0.25) * 2.0));
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let max = 1 << self.symbol_size;
                let (mut l, mut u) = ((l << 1) % max, (u << 1) % max + 1);
                let msb = 1 << (self.symbol_size - 1);
                let (l, u) = (l ^ msb, u ^ msb);

                return (Val::Int(l), Val::Int(u));
            }
        };
        unreachable!()
    }
    fn in_e3(&self, l: Val, u: Val) -> bool {
        if let Val::Float(l) = l {
            if let Val::Float(u) = u {
                return l < 0.5 && l > 0.25 && u > 0.5 && u < 0.75;
            }
        } else if let Val::Int(l) = l {
            if let Val::Int(u) = u {
                let msb1 = 0b10 << (self.symbol_size - 2);
                let msb2 = 0b01 << (self.symbol_size - 2);
                let mask = 0b11 << (self.symbol_size - 2);
                return (l & mask) == msb2 && (u & mask) == msb1;
            }
        };
        unreachable!()
    }

    fn update_l_and_u(&self, l: Val, u: Val, fx_l: Val, fx_r: Val) -> (Val, Val) {
        match l {
            Val::Float(l) => {
                if let Val::Float(u) = u {
                    if let Val::Float(fx_l) = fx_l {
                        if let Val::Float(fx_r) = fx_r {
                            let (l, u) = (l + (u - l) * fx_l, l + (u - l) * fx_r);
                            return (Val::Float(l), Val::Float(u));
                        }
                    }
                }
            }
            Val::Int(l) => {
                let total_cnt = self.total_cnt.unwrap();
                if let Val::Int(u) = u {
                    if let Val::Int(fx_l) = fx_l {
                        if let Val::Int(fx_r) = fx_r {
                            let (l, u) = (
                                l + (u - l + 1) * fx_l / total_cnt,
                                l + (u - l + 1) * fx_r / total_cnt - 1,
                            );
                            return (Val::Int(l), Val::Int(u));
                        }
                    }
                }
            }
        }
        panic!(
            "try to compute different type of Val l:{l:?}, r:{u:?}, fx_l:{fx_l:?}, fx_r:{fx_r:?}"
        );
    }
}

struct NumReader {
    symbol_size: usize,
    cur_val: Val,
    handler: BitIO,
}

impl NumReader {
    pub fn new(symbol_size: usize, mut handler: BitIO, is_int: bool) -> Self {
        let cur_val = if !is_int {
            let mut cur_val = 0f64;
            let mut base = 1.0;
            for _ in 0..symbol_size {
                base = base / 2.0;
                if handler.read_bit_front().unwrap() {
                    cur_val += base
                }
            }
            Val::Float(cur_val)
        } else {
            let mut cur_val = 0;
            for i in (0..symbol_size).rev() {
                if handler.read_bit_front().unwrap() {
                    cur_val += (1 << i)
                }
            }
            Val::Int(cur_val)
        };
        Self {
            symbol_size,
            cur_val,
            handler,
        }
    }
    pub fn cur_val(&self) -> Val {
        self.cur_val
    }
    pub fn cur_bit(&self) -> String {
        let res = match self.cur_val {
            Val::Float(val) => {
                let mut val = val;
                let mut base = 0.5;
                let mut res = "".to_string();
                for _ in 0..self.symbol_size {
                    if val >= base {
                        res.push('1');
                        val -= base;
                    } else {
                        res.push('0');
                    }
                    base /= 2.0;
                }
                res
            }
            Val::Int(val) => {
                let s = format!("{:b}", val);
                let padding_size = self.symbol_size - s.len();
                let mut prefix = vec!['0'; padding_size];
                prefix.extend(s.chars());
                String::from_iter(prefix)
            }
        };
        res
    }
    pub fn next(&mut self) -> Val {
        self.cur_val = match self.cur_val {
            Val::Float(f) => {
                let mut f = f;
                f *= 2.0;
                if f > 1.0 {
                    f -= 1.0;
                }
                if self.handler.read_bit_front().unwrap() {
                    let mut base = 1.0;
                    for _ in 0..self.symbol_size {
                        base = base / 2.0;
                    }
                    f += base;
                }
                Val::Float(f)
            }
            Val::Int(v) => {
                let mut v = v;
                let max = 1 << self.symbol_size;
                v <<= 1;
                v = v % max;
                if self.handler.read_bit_front().unwrap() {
                    v += 1;
                }
                Val::Int(v)
            }
        };
        self.cur_val
    }
    pub fn is_empty(&self) -> bool {
        self.handler.is_empty()
    }
}

struct Fx {
    symbol_to_index: HashMap<char, usize>,
    index_to_symbol: HashMap<usize, char>,
    freq: HashMap<usize, Val>,
}

impl Add for Val {
    type Output = Val;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            Val::Float(r) => match self {
                Val::Float(l) => Val::Float(l + r),
                Val::Int(l) => {
                    if r == 0.0 {
                        Val::Int(l)
                    } else if l == 0 {
                        Val::Float(r)
                    } else {
                        panic!("cannot add {self:?} and {rhs:?}")
                    }
                }
            },
            Val::Int(r) => match self {
                Val::Float(l) => {
                    if r == 0 {
                        Val::Float(l)
                    } else if l == 0.0 {
                        Val::Int(r)
                    } else {
                        panic!("cannot add {self:?} and {rhs:?}")
                    }
                }
                Val::Int(l) => Val::Int(r + l),
            },
        }
    }
}

impl Fx {
    pub fn new(freq: &Vec<(char, Val)>) -> Fx {
        let mut cur_freq = Val::Int(0);
        let mut fx_freq = HashMap::new();
        let mut symbol_to_index = HashMap::new();
        let mut index_to_symbol = HashMap::new();
        let mut is_int = false;

        for (i, (c, f)) in freq.iter().enumerate() {
            match f {
                Val::Int(_) => {
                    is_int = true;
                }
                _ => {}
            }
            symbol_to_index.insert(*c, i + 1);
            index_to_symbol.insert(i + 1, *c);

            cur_freq = f.add(cur_freq);
            fx_freq.insert(i + 1, cur_freq);
        }
        fx_freq.insert(0, if is_int { Val::Int(0) } else { Val::Float(0.0) });

        Fx {
            symbol_to_index,
            index_to_symbol,
            freq: fx_freq,
        }
    }
    pub fn get_val(&self, c: char) -> Val {
        *self
            .freq
            .get(self.symbol_to_index.get(&c).unwrap())
            .unwrap()
    }
    pub fn get_prev_val(&self, c: char) -> Val {
        *self
            .freq
            .get(&(self.symbol_to_index.get(&c).unwrap() - 1))
            .unwrap()
    }
    pub fn find_char(&self, f: Val, l: Val, u: Val, total_cnt: Option<usize>) -> char {
        let val = match f {
            Val::Float(f) => {
                if let Val::Float(l) = l {
                    if let Val::Float(u) = u {
                        Val::Float((f - l) / (u - l))
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Val::Int(f) => {
                let total_cnt = total_cnt.unwrap();
                if let Val::Int(l) = l {
                    if let Val::Int(u) = u {
                        Val::Int(((f - l + 1) * total_cnt - 1) / (u - l + 1))
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
        };
        for i in 1..self.freq.len() {
            let cur_f = *self.freq.get(&i).unwrap();
            if cur_f > val {
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
    fn test_encode_float() {
        let freq = vec![('a', 0.8f64), ('b', 0.02f64), ('c', 0.18f64)];
        let mut codec = Codec::new_with_float_freq(8, freq, None);
        let input = "acbaa";
        let handler = codec.encode(input);
        println!("{handler:?}");
        let res = codec.decode(handler);
        println!("{res}");
        assert_eq!(res, input);
    }
    #[test]
    fn test_encode_int() {
        let freq = vec![('a', 40), ('b', 1), ('c', 9)];
        let mut codec = Codec::new_with_int_freq(8, freq, Some(50));
        let input = "acbaabbbbbbbbbbbbbabcbab";
        let handler = codec.encode(input);
        println!("{handler:?}");
        let res = codec.decode(handler);
        println!("{res}");
        assert_eq!(res, input);
    }
}
