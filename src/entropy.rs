use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use serde::Serialize;

use crate::histogram::Histogram;

/// Calculate the entropy of a sequence of items
pub fn data_entropy<T, I>(iterable: I) -> f64
where
    T: Copy + Eq + Hash,
    I: IntoIterator<Item = T>,
{
    let mut counts = HashMap::new();
    let mut total_count = 0.0;

    // Count the occurrences of each item
    for item in iterable {
        *counts.entry(item).or_insert(0) += 1;
        total_count += 1.0;
    }

    // Calculate entropy
    let entropy = counts.values().fold(0.0, |acc, &count| {
        let probability = count as f64 / total_count;
        acc - (probability * probability.log2()) // log2 for information entropy
    });

    entropy
}

pub fn histogram_entropy<T>(histogram: &Histogram<T>) -> f64
where
    T: Copy + Eq + Hash + Ord + Serialize + Display,
{
    let total_count = histogram.total_count() as f64;
    let entropy = histogram
        .counts()
        .values()
        .fold(0.0, |acc, &count| {
            let probability = count as f64 / total_count;
            acc - (probability * probability.log2()) // log2 for information entropy
        });

    entropy
}