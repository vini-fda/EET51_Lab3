use std::io::Write;
use eet51_lab3::{huffman::{huffman_encode, weighted_path_length, huffman_tree}, histogram::Histogram, entropy::{histogram_entropy, data_entropy}, golomb::encode::custom_encode};
use image::GrayImage;
use serde::Serialize;
use ndarray::Array2;
use std::env;

#[derive(Serialize)]
struct PixelFrequency {
    pixel: u8,
    frequency: f32,
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

fn complete_tasks(img: &GrayImage, img_name: &str) {
    // Task A (Item 2): calculate the relative frequency of each pixel value in the image
    let histogram = Histogram::from_iter(img.pixels().map(|p| p[0]));
    // save to csv
    let path = format!("{}.csv", img_name);
    histogram.to_csv(&path, 0, 255).unwrap();
    // Task B (Item 3): calculate the entropy of a pixel modeled as a random variable
    let entropy = histogram_entropy(&histogram);
    println!("H(I): {}", entropy);
    // Task C (Item 4): calculate the prediction error matrix
    let prediction_err = prediction_err_matrix(img);
    // Task D (Item 5): calculate the entropy of the prediction error matrix
    let matrix_entropy = data_entropy(&prediction_err);
    println!("H(P): {}", matrix_entropy);
    // Task E (Item 6): reconstruct the image from the prediction error matrix
    let reconstructed_image = reconstruct_image_from_pred_err_matrix(&prediction_err);
    //verify that they are equal
    verify_equality_imgs(img, &reconstructed_image);

    // Task F (Item 7): create a new image from the absolute value of the prediction error matrix
    let abs_prediction_err = prediction_err.mapv(|x| x.abs());

    // Task G (Item 8): calculate the entropy of the sign of the prediction error matrix
    // and of the absolute value of the prediction error matrix
    let abs_prediction_err_entropy = data_entropy(&abs_prediction_err);
    let pixel_signs = prediction_err.iter().map(|&x| x < 0);
    let pixel_signs_entropy = data_entropy(pixel_signs);
    println!("H(|P|): {}", abs_prediction_err_entropy);
    println!("H(sgn(P)): {}", pixel_signs_entropy);

    // Task H (Item 9): Use the Custom Golomb encoding function to encode the prediction error matrix
    println!("================");
    println!("Custom encoding w/ Golomb");
    println!("================");
    let img_pixels = img.width() * img.height();
    let custom_encoded = custom_encode(&prediction_err);
    println!("Original image size: {} bits", img_pixels * 9);
    println!("Encoded image size: {} bits", custom_encoded.bits());
    println!("Compression ratio of P: {}", (img_pixels * 9) as f32 / custom_encoded.bits() as f32);
    println!("m: {}", custom_encoded.m);

    // This is not a task, but we will decode the encoded bytes and verify that they are equal
    let custom_decoded = custom_encoded.decode();
    verify_equality_arrays(&prediction_err, &custom_decoded);

    // Comparison with Huffman encoding
    println!("================");
    println!("Huffman encoding");
    println!("================");
    // encode original image
    let encoded = huffman_encode(img.pixels().map(|p| p[0]));
    // print the bits
    println!("Original image size (I): {} bits", img_pixels * 8);
    println!("Encoded image size (I): {} bits", encoded.len());

    // Compression ratio
    println!("Compression ratio (I): {}", (img_pixels * 8) as f32 / encoded.len() as f32);

    let weighted_path_length_orig = weighted_path_length(img.pixels().map(|p| p[0]));
    println!("Weighted path length (I): {}", weighted_path_length_orig);

    // encode prediction error matrix
    let encoded = huffman_encode(prediction_err.iter());
    // print the bits
    println!("Original image size (P): {} bits", prediction_err.len() * 9);
    println!("Encoded image size (P): {} bits", encoded.len());

    // Compression ratio
    println!("Compression ratio of (P): {}", (prediction_err.len() * 9) as f32 / encoded.len() as f32);

    let weighted_path_length_pred_err = weighted_path_length(prediction_err.iter());
    println!("Weighted path length of (P): {}", weighted_path_length_pred_err);

    let huffman_tree = huffman_tree(prediction_err.iter());
    // save to a file 
    let path = format!("{}_huffman_tree.dot", img_name);
    // use fmt::Display to print the tree to file
    let mut file = std::fs::File::create(path).unwrap();
    write!(file, "{}", huffman_tree).unwrap();
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

fn verify_equality_arrays(a: &Array2<i32>, b: &Array2<i32>) {
    if a.dim() != b.dim() {
        panic!("Arrays are not equal: dimensions are different");
    }
    for (x, y) in a.indexed_iter() {
        if *y != b[[x.0, x.1]] {
            panic!("Arrays are not equal at ({}, {}): {} != {}", x.0, x.1, *y, b[[x.0, x.1]]);
        }
    }
    println!("Arrays are equal");
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