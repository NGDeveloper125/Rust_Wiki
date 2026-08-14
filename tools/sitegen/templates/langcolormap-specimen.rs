#![allow(dead_code)]

use std::collections::HashMap;

const MAX_ENTRIES: usize = 128;
static UNIT: char = '·';

/// Counts words, keyed by label.
#[derive(Debug, Clone)]
pub struct Counter<'a, T> {
    label: &'a str,
    counts: HashMap<String, T>,
    ratio: f64,
}

pub enum Mode {
    Fast,
    Careful,
}

pub trait Summarize {
    fn summary(&self) -> String;
}

impl<'a, T: Default> Counter<'a, T> {
    pub fn new(label: &'a str) -> Self {
        Counter {
            label,
            counts: HashMap::new(),
            ratio: 0.5,
        }
    }

    fn record(&mut self, word: &str) -> Option<usize> {
        if self.counts.len() >= MAX_ENTRIES {
            return None;
        }
        self.counts.insert(word.to_string(), T::default());
        Some(self.counts.len())
    }

    fn first_len(&self) -> Option<usize> {
        let len = self.counts.keys().next()?.len();
        Some(len)
    }
}

impl<'a, T: Default> Summarize for Counter<'a, T> {
    fn summary(&self) -> String {
        format!("{} {UNIT} {}", self.label, self.counts.len())
    }
}

fn tally(values: &[usize]) -> usize {
    values.iter().copied().sum()
}

fn main() {
    let mut counter = Counter::<String>::new("words");
    let added = counter.record("ferris").unwrap_or(0);
    let doubled: Vec<usize> = [1, 2, 3].iter().map(|n| n * 2).collect();
    let total = tally(&doubled);
    let verbose: bool = true;
    let mode = Mode::Fast;
    if let Mode::Careful = mode { return; }

    if verbose && added > 0 {
        println!("{} {}", counter.summary(), total);
    }

    unsafe {
        let raw = &UNIT as *const char;
        let _ = *raw;
    }

    'outer: for value in &doubled {
        if *value == 4 { break 'outer; }
    }
}