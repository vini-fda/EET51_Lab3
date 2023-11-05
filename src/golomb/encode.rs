use ndarray::Array2;

pub struct CustomGolombEncodedImage {
    pub m: u8,
    pub encoded_bits: Vec<u8>,
    pub shape: (usize, usize),
}

impl CustomGolombEncodedImage {
    pub fn bits(&self) -> usize {
        self.encoded_bits.len()
    }

    pub fn decode(&self) -> Array2<i32> {
        custom_decode(self)
    }
}


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
pub fn custom_encode(matrix: &Array2<i32>) -> CustomGolombEncodedImage
{
    let abs_value = matrix.mapv(|x| x.abs());
    let abs_value_len = abs_value.len();
    let mut mean = 0.0;
    for n in abs_value {
        mean += n as f32;
    }
    mean /= abs_value_len as f32;
    // Golomb encoding parameter
    let mut m = 1u8;
    let mut b = 0;
    while (m as f32) < mean / 2.0 {
        m *= 2;
        b += 1;
    }
    let mut encoded_bits: Vec<u8> = Vec::new();

    for &v in matrix {
        // Quotient and Remainder Calculation
        let v_abs = v.unsigned_abs() as u8;
        let v_sign = v < 0;
        let q = v_abs / m;
        let r = v_abs % m;

        // Add the sign bit
        encoded_bits.push(v_sign as u8);
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
    // println!("Number of bits: {}", encoded_bits.len());
    let shape = matrix.shape();
    let shape = (shape[0], shape[1]);
    CustomGolombEncodedImage { m, encoded_bits, shape }
}

fn custom_decode(data: &CustomGolombEncodedImage) -> Array2<i32> {
    let mut decoded_pixels: Vec<i32> = Vec::new();
    let m = data.m;
    let bits = &data.encoded_bits;

    let mut b = 0;
    while (1 << b) < m {
        b += 1;
    }

    let mut i = 0;

    while i < bits.len() {
        let sign = bits[i] == 1;
        i += 1;
        let mut q = 0;
        while bits[i] == 0 {
            q += 1;
            i += 1;
        }
        i += 1;
        let mut r = 0;
        for j in 0..b {
            r |= bits[i + j] << (b - j - 1);
        }
        i += b;
        if r >= m {
            r -= m;
        }
        let v = (q * m + r) as i32;
        decoded_pixels.push(if sign { -v } else { v });
    }
    Array2::from_shape_vec(data.shape, decoded_pixels).unwrap()
}
