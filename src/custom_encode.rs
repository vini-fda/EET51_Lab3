// custom encoding based on golomb encoding
// we store the sign bit separately
// and the absolute value is an unsigned 8-bit integer

#[derive(Clone, Copy)]
pub struct CustomPixel {
    pub sign: bool,
    pub value: u8,
}

pub fn custom_encode(data: &[CustomPixel]) -> (u8, Vec<u8>) {
    let mut mean = 0.0;
    // mean of the values
    for &n in data {
        mean += n.value as f32;
    }
    mean /= data.len() as f32;
    let mut m = 1u8;
    while (m as f32) < mean / 2.0 {
        m *= 2;
    }
    (m, custom_encode_inner(data, m))
}

pub fn custom_encode_inner(data: &[CustomPixel], m: u8) -> Vec<u8> {
    let mut encoded_bits: Vec<u8> = Vec::new();
    // we assume m is a power of 2
    let mut b = 0;
    while (1 << b) < m {
        b += 1;
    }

    for &pixel in data {
        // Quotient and Remainder Calculation
        let q = pixel.value / m;
        let r = pixel.value % m;

        // Add the sign bit
        encoded_bits.push(pixel.sign as u8);
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
    encoded_bits
}

pub fn custom_decode(data: &[u8], m: u8) -> Vec<CustomPixel> {
    let mut decoded_pixels: Vec<CustomPixel> = Vec::new();

    let mut b = 0;
    while (1 << b) < m {
        b += 1;
    }

    let mut i = 0;

    while i < data.len() {
        let sign = data[i] == 1;
        i += 1;
        let mut q = 0;
        while data[i] == 0 {
            q += 1;
            i += 1;
        }
        i += 1;
        let mut r = 0;
        for j in 0..b {
            r |= data[i + j] << (b - j - 1);
        }
        i += b;
        if r >= m {
            r -= m;
        }
        decoded_pixels.push(CustomPixel { sign, value: q * m + r });
    }
    decoded_pixels
}