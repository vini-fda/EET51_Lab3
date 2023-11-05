use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use serde::Serialize;

#[derive(Serialize)]
struct CsvRow<T> {
    pixel: T,
    frequency: f64,
}

pub struct Histogram<T> {
    counts: HashMap<T, usize>,
    total_count: usize,
}

// Implement methods for Histogram
impl<T> Histogram<T>
where
    T: Hash + Eq + Copy + Ord + fmt::Display + Serialize,
{
    // Create a new Histogram.
    pub fn new() -> Self {
        Histogram {
            counts: HashMap::new(),
            total_count: 0,
        }
    }

    // Add items to the Histogram.
    pub fn add(&mut self, value: T) {
        let count = self.counts.entry(value).or_insert(0);
        *count += 1;
        self.total_count += 1;
    }

    // Print absolute frequencies.
    pub fn print_absolute(&self) {
        for (value, &count) in &self.counts {
            println!("{}: {}", value, count);
        }
    }

    // Print relative frequencies.
    pub fn print_relative(&self) {
        for (value, &count) in &self.counts {
            println!("{}: {:.2}", value, count as f64 / self.total_count as f64);
        }
    }

    // Print the full range, including 0-frequency elements.
    pub fn print_full_range(&self, min: T, max: T)
    where
        T: std::ops::Add<Output = T> + From<u8> + Copy,
    {
        let mut current = min;
        while current <= max {
            let count = *self.counts.get(&current).unwrap_or(&0);
            println!("{}: {}", current, count);
            // Increment current. Note: this requires T to be able to be constructed from a u8.
            current = current + T::from(1u8);
        }
    }

    // Method to serialize the histogram to a CSV file, including 0-frequencies.
    pub fn to_csv(&self, path: &str, min: T, max: T) -> Result<(), Box<dyn Error>>
    where
        T: std::ops::Add<Output = T> + From<u8> + Copy,
    {
        let mut writer = csv::Writer::from_path(path)?;

        // Write the header
        writer.write_record(["element", "frequency"])?;

        // Iterate over the full range and write the frequency or 0.0 if not present
        let mut current = min;
        while current <= max {
            let count = self.counts.get(&current).copied().unwrap_or(0);
            let frequency = count as f64 / self.total_count as f64;

            writer.serialize(CsvRow {
                pixel: current,
                frequency,
            })?;

            if current == max {
                break;
            }
            // Increment current. Note: this requires T to be able to be constructed from a u8.
            current = current + T::from(1u8);
        }

        writer.flush()?;
        Ok(())
    }

    // Get the total count of all items in the Histogram.
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn counts(&self) -> &HashMap<T, usize> {
        &self.counts
    }
}

impl<T> Default for Histogram<T>
where
    T: Hash + Eq + Copy + Ord + fmt::Display + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

// Implementing Display to show the histogram counts in a formatted way.
impl<T> fmt::Display for Histogram<T>
where
    T: Hash + Eq + Copy + Ord + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut entries: Vec<(&T, &usize)> = self.counts.iter().collect();
        entries.sort_by_key(|entry| entry.0);

        for (value, count) in entries {
            writeln!(f, "{}: {}", value, count)?;
        }

        Ok(())
    }
}

// Histogram from iterator  of T
impl<T> FromIterator<T> for Histogram<T>
where
    T: Hash + Eq + Copy + Ord + fmt::Display + Serialize,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut histogram = Histogram::new();
        for item in iter {
            histogram.add(item);
        }
        histogram
    }
}