pub mod fs_utility;

pub fn le_bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut result = 0;

    for (idx, val) in bytes[0..8].iter().enumerate() {
        result |= (*val as u64) << (idx * 8);
    }

    result
}

pub fn le_bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut result = 0;

    for (idx, val) in bytes[0..4].iter().enumerate() {
        result |= (*val as u32) << (idx * 8);
    }

    result
}

pub fn le_bytes_to_u64_padded(bytes: &[u8]) -> u64 {
    let mut result = 0;

    let mut idx = 0;

    while idx < bytes.len() {
        result |= (bytes[idx] as u64) << (idx * 8);
        idx += 1;
    }

    result
}
