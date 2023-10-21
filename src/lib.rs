pub mod huffman_encode;

use image::GrayImage;

/*
    The Golomb encoding algorithm is a lossless data compression method used for entropy encoding. 
    It’s particularly effective when applied to data that exhibit geometrically distributed characteristics.
    Below is a straightforward explanation of the algorithm:

    1. **Parameter Selection:**
       - Choose a positive integer parameter `M`. The choice of `M` affects the efficiency of the encoding.
       - `M` should be chosen based on the characteristics of the input data.
       - In practice, it is convenient to choose `M` as a power of 2.
       - In this case, we will use `k` as the smallest integer greater than or equal to log2(mu/2), 
         where `mu` is the mean of the input data, and `M = 2^k`

    2. **Encoding a Non-negative Integer:**
       - Divide the integer `x` by the parameter `M`, obtaining a quotient `q` and a remainder `r`.
       - `q` is the result of the integer division, and `r` is the remainder: `x = q * M + r`

    3. **Unary Coding of the Quotient:**
       - Encode the quotient `q` in unary code. Unary coding involves writing `q` zeroes followed by a one: `0^q 1`

    4. **Binary Coding of the Remainder:**
       - Encode the remainder `r` in binary. The binary code can be of fixed or variable length depending on the chosen `M`.
       - If `M` is a power of 2, simple binary encoding is used. Otherwise, some variations like truncated binary encoding may be applied.

    5. **Concatenation:**
       - Concatenate the unary code of `q` and the binary code of `r` to get the final Golomb code.

    Example:
        Suppose we choose `M = 3` and we want to encode the number `10`.
        - Quotient and Remainder: `10 = 3 * 3 + 1`
        - Unary code of quotient: `0001`
        - Binary code of remainder: `01`
        - Concatenated Golomb code: `000101`

    Decoding involves reversing these steps, extracting the quotient and remainder from the encoded bitstream, 
    and reconstructing the original integer using the formula `x = q * M + r`.

    Note: Golomb coding is efficient for data where lower values are more probable than higher values, 
    making it suitable for applications like image compression and run-length encoding.
*/
pub fn golomb_encode(data: &[u8]) -> (u8, Vec<u8>) {
    let mut mean = 0.0;
    for &n in data {
        mean += n as f32;
    }
    mean /= data.len() as f32;

    let mut m = 1u8;
    while (m as f32) < mean / 2.0 {
        m *= 2;
    }

    (m, golomb_encode_inner(data, m))
}


fn golomb_encode_inner(data: &[u8], m: u8) -> Vec<u8> {
    let mut encoded_bits: Vec<u8> = Vec::new();
    // we assume m is a power of 2
    let mut b = 0;
    while (1 << b) < m {
        b += 1;
    }

    for &n in data {
        // Quotient and Remainder Calculation
        let q = n / m;
        let r = n % m;

        // Unary Encoding of the Quotient
        // add q values of 0
        encoded_bits.resize(encoded_bits.len() + q as usize, 0);
        // add a 1
        encoded_bits.push(1);

        // Truncated Binary Encoding of the Remainder
        if r < m {
            for i in (0..b).rev() {
                encoded_bits.push((r >> i) & 1);
            }
        } else {
            let adjusted_r = r + m;
            for i in (0..=b).rev() {
                encoded_bits.push((adjusted_r >> i) & 1);
            }
        }
    }
    println!("Number of bits: {}", encoded_bits.len());
    pack_bits(&encoded_bits)
}

pub fn print_as_bits(data: &[u8]) {
    for &bit in data {
        print!("{}", bit);
    }
    println!();
}

fn pack_bits(encoded_bits: &[u8]) -> Vec<u8> {
    let mut packed_bytes = Vec::new();
    let mut current_byte = 0u8;
    let mut bit_count = 0;

    for bit in encoded_bits {
        current_byte = (current_byte << 1) | bit;
        bit_count += 1;

        if bit_count == 8 {
            packed_bytes.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }

    // If there are remaining bits, pad and push them
    if bit_count != 0 {
        current_byte <<= 8 - bit_count;
        packed_bytes.push(current_byte);
    }

    packed_bytes
}

/// Convert image to bytes
pub fn image_to_bytes(image: &GrayImage) -> Vec<u8> {
    image.pixels().map(|p| p[0]).collect()
}

/// Convert bytes to image
pub fn bytes_to_image(bytes: &[u8], width: u32, height: u32) -> GrayImage {
    let mut image = GrayImage::new(width, height);
    for (i, pixel) in image.pixels_mut().enumerate() {
        pixel[0] = bytes[i];
    }
    image
}

pub fn golomb_decode(m: u8, data: &[u8]) -> Vec<u8> {
    let mut decoded_bits: Vec<u8> = Vec::new();
    // we assume m is a power of 2
    let mut b = 0;
    while (1 << b) < m {
        b += 1;
    }

    let mut bit_index = 0;
    while bit_index < data.len() * 8 {
        // Unary Decoding of the Quotient
        let mut q = 0;
        while data[bit_index / 8] & (1 << (7 - (bit_index % 8))) == 0 {
            q += 1;
            bit_index += 1;
            if bit_index >= data.len() * 8 {
                return decoded_bits;
            }
        }
        bit_index += 1;

        // Truncated Binary Decoding of the Remainder
        let mut r = 0;
        for _ in 0..b {
            r <<= 1;
            if data[bit_index / 8] & (1 << (7 - (bit_index % 8))) != 0 {
                r |= 1;
            }
            bit_index += 1;
            if bit_index >= data.len() * 8 {
                return decoded_bits;
            }
        }

        // Reconstruct the original integer
        let n = q * m + r;
        decoded_bits.push(n);
    }

    decoded_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golomb_encode() {
        let data = vec![21, 21, 21, 21];
        let encoded = golomb_encode_inner(&data, 8);
        // The expected binary sequence for 21 is 001101
        // So, the packed bytes should be 001101|001101|001101|001101 = 00110100 11010011 01001101
        let expected = vec![0b00110100, 0b11010011, 0b01001101];

        assert_eq!(encoded, expected, "Golomb encoding failed to provide the expected output.");
    }

    #[test]
    fn test_golomb_encode_2() {
        let data = vec![21];
        let encoded = golomb_encode_inner(&data, 8);
        // The expected binary sequence for 21 is 001101
        // So, the packed bytes should be 001101 -> 00110100
        let expected = vec![0b00110100];

        assert_eq!(encoded, expected, "Golomb encoding failed to provide the expected output.");
    }
}