use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HuffmanNode<T>
where
    T: Ord + Copy,
{
    Internal {
        left: Box<HuffmanNode<T>>,
        right: Box<HuffmanNode<T>>,
    },
    Leaf {
        value: T,
        frequency: u32,
    },
}

impl<T> fmt::Display for HuffmanNode<T>
where
    T: fmt::Debug + Ord + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "digraph HuffmanTree {{")?;
        self.dot_helper(f, 0)?;
        write!(f, "}}")
    }
}

impl<T> HuffmanNode<T>
where
    T: fmt::Debug + Ord + Copy,
{
    fn dot_helper(&self, f: &mut fmt::Formatter, id: usize) -> fmt::Result {
        match self {
            HuffmanNode::Internal { left, right } => {
                let left_id = 2 * id + 1;
                let right_id = 2 * id + 2;

                writeln!(f, "    node{} [label=\"\"];", id)?;
                writeln!(f, "    node{} -> node{};", id, left_id)?;
                writeln!(f, "    node{} -> node{};", id, right_id)?;

                left.dot_helper(f, left_id)?;
                right.dot_helper(f, right_id)?;
            }
            HuffmanNode::Leaf { value, frequency } => {
                writeln!(f, "    node{} [label=\"{:?} ({})\"];", id, value, frequency)?;
            }
        }
        Ok(())
    }
}

// implement Ord
impl<T> Ord for HuffmanNode<T>
where
    T: Ord + Copy,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (
                HuffmanNode::Internal { left: _, right: _ },
                HuffmanNode::Leaf {
                    value: _,
                    frequency: _,
                },
            ) => std::cmp::Ordering::Less,
            (
                HuffmanNode::Leaf {
                    value: _,
                    frequency: _,
                },
                HuffmanNode::Internal { left: _, right: _ },
            ) => std::cmp::Ordering::Greater,
            (
                HuffmanNode::Internal { left: _, right: _ },
                HuffmanNode::Internal { left: _, right: _ },
            ) => std::cmp::Ordering::Equal,
            (
                HuffmanNode::Leaf {
                    value: _,
                    frequency: freq1,
                },
                HuffmanNode::Leaf {
                    value: _,
                    frequency: freq2,
                },
            ) => freq1.cmp(freq2),
        }
    }
}

// implement PartialOrd
impl<T> PartialOrd for HuffmanNode<T>
where
    T: Ord + Copy,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn build_histogram<T, I>(data: I) -> BTreeMap<T, u32>
where
    T: Ord + Copy + Hash,
    I: Iterator<Item = T>,
{
    let mut histogram = BTreeMap::new();

    for val in data {
        let count = histogram.entry(val).or_insert(0);
        *count += 1;
    }
    histogram
}

fn build_huffman_tree<T>(data: &BTreeMap<T, u32>) -> HuffmanNode<T>
where
    T: Ord + Copy,
{
    // Convert histogram into a priority queue of nodes
    let mut heap: BinaryHeap<_> = data
        .iter()
        .map(|(&value, &freq)| {
            (
                Reverse(freq),
                HuffmanNode::Leaf {
                    value,
                    frequency: freq,
                },
            )
        })
        .collect();

    // While there's more than one node left in the heap, pop out two,
    // merge them into an internal node, and push it back in.
    while heap.len() > 1 {
        let (freq1, left) = heap.pop().unwrap();
        let (freq2, right) = heap.pop().unwrap();
        let internal = HuffmanNode::Internal {
            left: Box::new(left),
            right: Box::new(right),
        };
        heap.push((Reverse(freq1.0 + freq2.0), internal));
    }

    // The remaining node is the root of the Huffman tree.
    heap.pop().unwrap().1
}

fn generate_codes<T>(
    node: &HuffmanNode<T>,
    current_code: VecDeque<u8>,
    codes: &mut HashMap<T, Vec<u8>>,
) where
    T: Ord + Copy + Hash,
{
    match node {
        HuffmanNode::Internal { left, right } => {
            let mut left_code = current_code.clone();
            left_code.push_back(0);
            generate_codes(left, left_code, codes);

            let mut right_code = current_code.clone();
            right_code.push_back(1);
            generate_codes(right, right_code, codes);
        }
        HuffmanNode::Leaf { value, .. } => {
            codes.insert(*value, current_code.into_iter().collect());
        }
    }
}

pub fn normalize_histogram(histogram: &BTreeMap<u8, u32>) -> BTreeMap<u8, f32> {
    let mut normalized = BTreeMap::new();
    let total: u32 = histogram.values().sum();
    for (&key, &value) in histogram.iter() {
        normalized.insert(key, value as f32 / total as f32);
    }
    normalized
}

pub fn huffman_encode<T, I>(data: I) -> Vec<u8>
where
    T: Ord + Copy + Hash,
    I: Iterator<Item = T> + Clone,
{
    let frequencies = build_histogram(data.clone());
    let tree = build_huffman_tree(&frequencies);
    let mut code_map = HashMap::new();
    generate_codes(&tree, VecDeque::new(), &mut code_map);

    let mut result = Vec::new();
    for byte in data {
        if let Some(code) = code_map.get(&byte) {
            result.extend(code.iter());
        }
    }
    result
}

pub fn huffman_tree<T, I>(data: I) -> HuffmanNode<T>
where
    T: Ord + Copy + Hash,
    I: Iterator<Item = T> + Clone,
{
    let frequencies = build_histogram(data.clone());
    build_huffman_tree(&frequencies)
}

pub fn weighted_path_length<T, I>(data: I) -> f64
where
    T: Ord + Copy + Hash,
    I: Iterator<Item = T> + Clone,
{
    let frequencies = build_histogram(data.clone());
    let tree = build_huffman_tree(&frequencies);
    let mut code_map = HashMap::new();
    generate_codes(&tree, VecDeque::new(), &mut code_map);

    let mut result = 0.0;
    // calculate the weighted path length = sum of (code length * frequency)
    let total = frequencies.values().sum::<u32>() as f64;
    for (byte, code) in code_map {
        result += code.len() as f64 * frequencies[&byte] as f64;
    }
    result / total
}