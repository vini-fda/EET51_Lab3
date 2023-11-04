use std::collections::BTreeMap;
use eet51_lab3::{golomb_encode, image_to_bytes, golomb_decode, bytes_to_image, huffman_encode::{huffman_encode, weighted_path_length}};
use image::{buffer::Pixels, Luma, GrayImage};
use serde::Serialize;
use ndarray::Array2;
use std::env;

fn grayscale_histogram(pixels: Pixels<'_, Luma<u8>>) -> BTreeMap<u8, u32> {
    let mut histogram = BTreeMap::new();
    // 0 - 255
    for i in 0..=255 {
        histogram.insert(i, 0);
    }
    for pixel in pixels {
        let count = histogram.entry(pixel.0[0]).or_insert(0);
        *count += 1;
    }
    histogram
}

fn matrix_histogram(matrix: &Array2<i32>) -> BTreeMap<i32, u32> {
    let mut histogram = BTreeMap::new();
    for pixel in matrix.iter() {
        let count = histogram.entry(*pixel).or_insert(0);
        *count += 1;
    }
    histogram
}

#[derive(Serialize)]
struct PixelFrequency {
    pixel: u8,
    frequency: f32,
}

fn save_to_csv(histogram: &BTreeMap<u8, f32>, filename: &str) {
    let mut wtr = csv::Writer::from_path(filename).unwrap();
    for (key, value) in histogram.iter() {
        wtr.serialize(PixelFrequency {
            pixel: *key,
            frequency: *value,
        }).unwrap();
    }
    wtr.flush().unwrap();
}

fn calculate_entropy<T>(histogram: &BTreeMap<T, f32>) -> f32 {
    let mut entropy = 0.0;
    for (_, &value) in histogram.iter() {
        if value > 0.0 {
            entropy -= value * value.log2();
        }
    }
    entropy
}

// create a new image with the same dimensions as the original
// for each pixel in the original image, find the corresponding pixel in the new image
// output(i, j) = input(i,j) -input(i-1,j) - input(i,j-1) + input(i-1,j-1)
fn prediction_err_matrix(original: &GrayImage) -> Array2<i32> {
    // take the dimensions (u32, u32) and convert to (usize, usize)
    let (width, height) = original.dimensions();
    let width = width as usize;
    let height = height as usize;
    let mut new_matrix = Array2::zeros((width, height));

    for x in 0..width {
        for y in 0..height {
            let new_value;
            let f = |x: usize, y: usize| -> i32 {
                if x >= width || y >= height {
                    0
                } else {
                    original.get_pixel(x as u32, y as u32)[0] as i32
                }
            };
            if x > 0 && y > 0 {
                new_value = f(x, y) - f(x - 1, y) - f(x, y - 1) + f(x - 1, y - 1);
            } else if x > 0 {
                new_value = f(x, y) - f(x - 1, y);
            } else if y > 0 {
                new_value = f(x, y) - f(x, y - 1);
            } else {
                new_value = f(x, y);
            }
            new_matrix[[x, y]] = new_value;
        }
    }
    new_matrix
}

fn reconstruct_image_from_pred_err_matrix(matrix: &Array2<i32>) -> GrayImage {
    let (width, height) = matrix.dim();
    let mut reconstructed_image = GrayImage::new(width as u32, height as u32);
    
    let f = |x: usize, y: usize| -> i32 {
        if x >= width || y >= height {
            0
        } else {
            matrix[[x, y]]
        }
    };



    for x in 0..width {
        for y in 0..height {
            let mut new_pixel;

            let g = |x: usize, y: usize| -> i32 {
                if x >= width || y >= height {
                    0
                } else {
                    reconstructed_image.get_pixel(x as u32, y as u32)[0] as i32
                }
            };

            // I(x,y) = \sum_{m=0}^{x} \sum_{n=0}^{y} P(m,n)
            // => I(x,y) = I(x-1,y) + I(x,y-1) - I(x-1,y-1) + P(x,y)
            if x > 0 && y > 0 {
                new_pixel = g(x - 1, y) + g(x, y - 1) - g(x - 1, y - 1) + f(x, y);
            } else if x > 0 {
                new_pixel = g(x - 1, y) + f(x, y);
            } else if y > 0 {
                new_pixel = g(x, y - 1) + f(x, y);
            } else {
                new_pixel = f(x, y);
            }
            //cap the value at 0 and 255
            if new_pixel < 0 {
                new_pixel = 0;
            } else if new_pixel > 255 {
                new_pixel = 255;
            }
            reconstructed_image.put_pixel(x as u32, y as u32, image::Luma([new_pixel as u8]));
        }
    }
    reconstructed_image
}

fn convert_to_grayscale_image(matrix: &Array2<i32>) -> GrayImage {
    let (width, height) = matrix.dim();
    let mut new_image = GrayImage::new(width as u32, height as u32);
    for x in 0..width {
        for y in 0..height {
            let pixel = matrix[[x, y]];
            //cap the value at 0 and 255
            let pixel = if pixel < 0 {
                0u8
            } else if pixel > 255 {
                255u8
            } else {
                pixel as u8
            };
            new_image.put_pixel(x as u32, y as u32, image::Luma([pixel]));
        }
    }
    new_image
}

fn complete_tasks(img: &GrayImage, img_name: &str) {
    // Task A (Item 2): calculate the relative frequency of each pixel value in the image
    let histogram = grayscale_histogram(img.pixels());
    let mut relative_freq = BTreeMap::new();
    for (&key, &value) in histogram.iter() {
        relative_freq.insert(key, value as f32 / (img.width() * img.height()) as f32);
    }
    let filename = format!("{}.csv", img_name);
    save_to_csv(&relative_freq, &filename);
    // Task B (Item 3): calculate the entropy of a pixel modeled as a random variable
    let entropy = calculate_entropy(&relative_freq);
    println!("Entropy: {}", entropy);
    // Task C (Item 4): calculate the prediction error matrix
    let prediction_err = prediction_err_matrix(img);
    println!("Prediction error matrix: {:?}", prediction_err);

    // Task D (Item 5): calculate the entropy of the prediction error matrix
    let matrix_histogram = matrix_histogram(&prediction_err);
    let mut matrix_relative_freq = BTreeMap::new();
    for (&key, &value) in matrix_histogram.iter() {
        matrix_relative_freq.insert(key, value as f32 / (img.width() * img.height()) as f32);
    }
    let matrix_entropy = calculate_entropy(&matrix_relative_freq);
    println!("Prediction error matrix entropy: {}", matrix_entropy);

    // Task E (Item 6): reconstruct the image from the prediction error matrix
    let reconstructed_image = reconstruct_image_from_pred_err_matrix(&prediction_err);
    //verify that they are equal
    verify_equality_imgs(img, &reconstructed_image);

    // Task F (Item 7): create a new image from the absolute value of the prediction error matrix
    let abs_prediction_err = prediction_err.mapv(|x| x.abs());
    let abs_reconstructed_image = convert_to_grayscale_image(&abs_prediction_err);

    // Task G (Item 8): calculate the entropy of the sign of the prediction error matrix
    let pixel_signs: Vec<bool> = prediction_err.iter().map(|&x| x < 0).collect();
    let mut pixel_signs_histogram = BTreeMap::new();
    for &sign in pixel_signs.iter() {
        let count = pixel_signs_histogram.entry(sign).or_insert(0);
        *count += 1;
    }
    let mut pixel_signs_relative_freq = BTreeMap::new();
    for (&key, &value) in pixel_signs_histogram.iter() {
        pixel_signs_relative_freq.insert(key, value as f32 / (img.width() * img.height()) as f32);
    }
    let pixel_signs_entropy = calculate_entropy(&pixel_signs_relative_freq);
    println!("Entropy of the sign of the prediction error matrix: {}", pixel_signs_entropy);

    // Task H (Item 9): Use the Golomb encoding function to encode the absolute value of the prediction error matrix
    let abs_bytes = image_to_bytes(&abs_reconstructed_image);
    let (m, encoded) = golomb_encode(&abs_bytes);
    // print the difference in bytes
    println!("Original image size: {} bytes", abs_bytes.len());
    println!("Encoded image size: {} bytes", encoded.len());
    // Compression ratio
    println!("Compression ratio: {}", abs_bytes.len() as f32 / encoded.len() as f32);
    // Percentage ratio of image size
    println!("Percentage of original image size: {:.2}%", encoded.len() as f32 / abs_bytes.len() as f32 * 100.0);

    // This is not a task, but we will decode the encoded bytes and verify that they are equal
    let decoded = golomb_decode(m, &encoded);
    let decoded_image = bytes_to_image(&decoded, img.width(), img.height());
    verify_equality_imgs(&abs_reconstructed_image, &decoded_image);

    // Comparison with Huffman encoding
    println!();
    println!("Huffman encoding");
    // encode original image
    let original_bytes = image_to_bytes(img);
    let encoded = huffman_encode(&original_bytes);
    // print the bits
    println!("Original image size: {} bits", original_bytes.len() * 8);
    println!("Encoded image size: {} bits", encoded.len());

    // weighted_path_length
    let wpl = weighted_path_length(&original_bytes);
    println!("Weighted path length: {}", wpl);

    // Compression ratio
    println!("Compression ratio: {}", (original_bytes.len() * 8) as f32 / encoded.len() as f32);

    // encode absolute value of prediction error matrix
    let encoded = huffman_encode(&abs_bytes);
    // print the bits
    println!("Original image size: {} bits", abs_bytes.len() * 8);
    println!("Encoded image size: {} bits", encoded.len());

    // weighted_path_length
    let wpl = weighted_path_length(&abs_bytes);
    println!("Weighted path length: {}", wpl);

    // Compression ratio
    println!("Compression ratio: {}", (abs_bytes.len() * 8) as f32 / encoded.len() as f32);

}

/// Verify that two grayscale images are equal
/// 1. They must have the same dimensions
/// 2. Each pixel must have the same value
/// 
/// We show the exact point where the images differ
/// with useful error messages
fn verify_equality_imgs(a: &GrayImage, b: &GrayImage) {
    if a.dimensions() != b.dimensions() {
        panic!("Images are not equal: dimensions are different");
    }
    for (x, y, pixel) in a.enumerate_pixels() {
        if pixel[0] != b.get_pixel(x, y)[0] {
            panic!("Images are not equal at ({}, {}): {} != {}", x, y, pixel[0], b.get_pixel(x, y)[0]);
        }
    }
    println!("Images are equal");
}



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide the path to the image.");
        return;
    }

    let img_path = &args[1];
    let img = image::open(img_path).unwrap().to_luma8();
    let img_name = img_path.split('/').last().unwrap().split('.').next().unwrap();
    complete_tasks(&img, img_name);
}