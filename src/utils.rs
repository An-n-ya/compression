use std::collections::HashMap;

pub fn freq_of_str(input: &str) -> Vec<(char, usize)> {
    let mut freq = HashMap::new();
    input.chars().for_each(|c| {
        freq.insert(c, freq.get(&c).unwrap_or(&0) + 1);
    });
    let freq: Vec<(char, usize)> = freq.iter().map(|(&k, &v)| (k, v)).collect();
    freq
}
