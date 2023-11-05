pub mod huffman;
pub mod entropy;
pub mod histogram;
pub mod golomb;

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
