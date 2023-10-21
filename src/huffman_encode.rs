use std::collections::{BTreeMap, BinaryHeap};
use std::collections::VecDeque;
use std::collections::HashMap;
use std::cmp::Reverse;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
enum HuffmanNode<T> where T: Ord + Copy {
    Internal { left: Box<HuffmanNode<T>>, right: Box<HuffmanNode<T>> },
    Leaf { value: T, frequency: u32 },
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
                writeln!(
                    f,
                    "    node{} [label=\"{:?} ({})\"];",
                    id, value, frequency
                )?;
            }
        }
        Ok(())
    }
}

// implement Ord
impl<T> Ord for HuffmanNode<T> where T: Ord + Copy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (HuffmanNode::Internal { left: _, right: _ }, HuffmanNode::Leaf { value: _, frequency: _ }) => std::cmp::Ordering::Less,
            (HuffmanNode::Leaf { value: _, frequency: _ }, HuffmanNode::Internal { left: _, right: _ }) => std::cmp::Ordering::Greater,
            (HuffmanNode::Internal { left: _, right: _ }, HuffmanNode::Internal { left: _, right: _ }) => std::cmp::Ordering::Equal,
            (HuffmanNode::Leaf { value: _, frequency: freq1 }, HuffmanNode::Leaf { value: _, frequency: freq2 }) => freq1.cmp(freq2),
        }
    }
}

// implement PartialOrd
impl<T> PartialOrd for HuffmanNode<T> where T: Ord + Copy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn build_histogram<T>(data: &[T]) -> BTreeMap<T, u32> 
where T: Ord + Copy
{
    let mut histogram = BTreeMap::new();

    for &val in data.iter() {
        let count = histogram.entry(val).or_insert(0);
        *count += 1;
    }
    histogram
}

fn build_huffman_tree(data: &BTreeMap<u8, u32>) -> HuffmanNode<u8> {
    // Convert histogram into a priority queue of nodes
    let mut heap: BinaryHeap<_> = data.iter()
        .map(|(&value, &freq)| {
            (Reverse(freq), HuffmanNode::Leaf { value, frequency: freq })
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

fn generate_codes<T>(node: &HuffmanNode<T>, current_code: VecDeque<u8>, codes: &mut HashMap<T, Vec<u8>>)
where T: Ord + Copy + std::hash::Hash
{
    match node {
        HuffmanNode::Internal { left, right } => {
            let mut left_code = current_code.clone();
            left_code.push_back(0);
            generate_codes(left, left_code, codes);

            let mut right_code = current_code.clone();
            right_code.push_back(1);
            generate_codes(right, right_code, codes);
        },
        HuffmanNode::Leaf { value, .. } => {
            codes.insert(*value, current_code.into_iter().collect());
        },
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

pub fn huffman_encode(data: &[u8]) -> Vec<u8> {
    let frequencies = build_histogram(data);
    let tree = build_huffman_tree(&frequencies);
    let mut code_map = HashMap::new();
    generate_codes(&tree, VecDeque::new(), &mut code_map);
    
    let mut result = Vec::new();
    for &byte in data.iter() {
        if let Some(code) = code_map.get(&byte) {
            result.extend(code.iter());
        }
    }
    result
}

// calculates the weighted path length of a Huffman code
// given a histogram of the frequencies of each symbol
pub fn weighted_path_length(data: &[u8]) -> f32
{
    let frequencies = build_histogram(data);
    let tree = build_huffman_tree(&frequencies);
    let mut code_map = HashMap::new();
    generate_codes(&tree, VecDeque::new(), &mut code_map);
    weighted_path_length_internal(&normalize_histogram(&frequencies), &code_map)
}


fn weighted_path_length_internal<T>(histogram_normalized: &BTreeMap<T, f32>, code_map: &HashMap<T, Vec<u8>>) -> f32
where T: Ord + Copy + std::hash::Hash
{
    let mut result = 0.0;
    for (symbol, freq) in histogram_normalized.iter() {
        if let Some(code) = code_map.get(symbol) {
            result += *freq * code.len() as f32;
        }
    }
    result
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_encode() {
        let input_str = "this is an example of a huffman tree";
        let _encoded_data = huffman_encode(input_str.as_bytes());
        // frequency of each byte in the input string
        println!("{:?}", build_histogram(input_str.chars().collect::<Vec<_>>().as_slice()));

        // Expected Huffman codes from the table.
        let expected_codes: HashMap<u8, Vec<u8>> = [
            (b' ', vec![1, 1, 1]),
            (b'a', vec![0, 1, 0]),
            (b'e', vec![0, 0, 0]),
            (b'f', vec![1, 1, 0, 1]),
            (b'h', vec![1, 0, 1, 0]),
            (b'i', vec![1, 0, 0, 0]),
            (b'm', vec![0, 1, 1, 1]),
            (b'n', vec![0, 0, 1, 0]),
            (b's', vec![1, 0, 1, 1]),
            (b't', vec![0, 1, 1, 0]),
            (b'l', vec![1, 1, 0, 0, 1]),
            (b'o', vec![0, 0, 1, 1, 0]),
            (b'p', vec![1, 0, 0, 1, 1]),
            (b'r', vec![1, 1, 0, 0, 0]),
            (b'u', vec![0, 0, 1, 1, 1]),
            (b'x', vec![1, 0, 0, 1, 0])
        ].iter().cloned().collect();

        // Compute Huffman codes for the input string.
        let tree = build_huffman_tree(&build_histogram(input_str.as_bytes()));
        println!("{}", tree);
        let mut code_map = HashMap::new();
        generate_codes(&tree, VecDeque::new(), &mut code_map);

        for (byte, code) in expected_codes.iter() {
            assert_eq!(code_map[byte], *code);
        }
    }
}