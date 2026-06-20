use super::*;

pub(in crate::arch::arm64::execute) fn simd_rotate_left_64_lanes(value: u128, shift: u32) -> u128 {
    let low = (value as u64).rotate_left(shift);
    let high = ((value >> 64) as u64).rotate_left(shift);
    ((high as u128) << 64) | low as u128
}

pub(in crate::arch::arm64::execute) fn simd_rotate_right_64_lanes(value: u128, shift: u32) -> u128 {
    let low = (value as u64).rotate_right(shift);
    let high = ((value >> 64) as u64).rotate_right(shift);
    ((high as u128) << 64) | low as u128
}

pub(in crate::arch::arm64::execute) fn simd_polynomial_mult(
    lhs: u128,
    rhs: u128,
    bits: usize,
) -> u128 {
    let mut out = 0u128;
    for bit in 0..bits {
        if ((rhs >> bit) & 1) != 0 {
            out ^= lhs << bit;
        }
    }
    out
}

pub(in crate::arch::arm64::execute) fn pack_u32_lanes(words: [u32; 4]) -> u128 {
    words
        .into_iter()
        .enumerate()
        .fold(0u128, |out, (lane, word)| {
            out | ((word as u128) << (lane * 32))
        })
}

pub(in crate::arch::arm64::execute) fn sm4_sub_word(value: u32) -> u32 {
    let mut out = 0u32;
    for byte in 0..4 {
        let input = ((value >> (byte * 8)) & 0xff) as usize;
        out |= (SM4_SBOX[input] as u32) << (byte * 8);
    }
    out
}

pub(in crate::arch::arm64::execute) fn sm4_linear_transform(value: u32) -> u32 {
    value
        ^ value.rotate_left(2)
        ^ value.rotate_left(10)
        ^ value.rotate_left(18)
        ^ value.rotate_left(24)
}

pub(in crate::arch::arm64::execute) fn sm4_key_transform(value: u32) -> u32 {
    value ^ value.rotate_left(13) ^ value.rotate_left(23)
}

pub(in crate::arch::arm64::execute) fn aes_sub_shift_round(value: u128, decrypt: bool) -> u128 {
    let bytes = aes_state_bytes(value);
    let mut out = [0u8; 16];
    for (index, slot) in out.iter_mut().enumerate() {
        let source_index = if decrypt {
            (index * 13) & 0xF
        } else {
            (index * 5) & 0xF
        };
        let table = if decrypt { &AES_INV_SBOX } else { &AES_SBOX };
        *slot = table[bytes[source_index] as usize];
    }
    aes_state_from_bytes(out)
}

pub(in crate::arch::arm64::execute) fn aes_mix_columns(value: u128, inverse: bool) -> u128 {
    let bytes = aes_state_bytes(value);
    let mut out = [0u8; 16];
    for column in 0..4 {
        let base = column * 4;
        let a = bytes[base];
        let b = bytes[base + 1];
        let c = bytes[base + 2];
        let d = bytes[base + 3];
        if inverse {
            out[base] = aes_mul(a, 14) ^ aes_mul(b, 11) ^ aes_mul(c, 13) ^ aes_mul(d, 9);
            out[base + 1] = aes_mul(a, 9) ^ aes_mul(b, 14) ^ aes_mul(c, 11) ^ aes_mul(d, 13);
            out[base + 2] = aes_mul(a, 13) ^ aes_mul(b, 9) ^ aes_mul(c, 14) ^ aes_mul(d, 11);
            out[base + 3] = aes_mul(a, 11) ^ aes_mul(b, 13) ^ aes_mul(c, 9) ^ aes_mul(d, 14);
        } else {
            out[base] = aes_mul(a, 2) ^ aes_mul(b, 3) ^ c ^ d;
            out[base + 1] = a ^ aes_mul(b, 2) ^ aes_mul(c, 3) ^ d;
            out[base + 2] = a ^ b ^ aes_mul(c, 2) ^ aes_mul(d, 3);
            out[base + 3] = aes_mul(a, 3) ^ b ^ c ^ aes_mul(d, 2);
        }
    }
    aes_state_from_bytes(out)
}

pub(in crate::arch::arm64::execute) fn aes_state_bytes(value: u128) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = ((value >> (index * 8)) & 0xFF) as u8;
    }
    bytes
}

pub(in crate::arch::arm64::execute) fn aes_state_from_bytes(bytes: [u8; 16]) -> u128 {
    let mut value = 0u128;
    for (index, byte) in bytes.iter().enumerate() {
        value |= (*byte as u128) << (index * 8);
    }
    value
}

pub(in crate::arch::arm64::execute) fn aes_mul(mut value: u8, mut factor: u8) -> u8 {
    let mut out = 0u8;
    while factor != 0 {
        if (factor & 1) != 0 {
            out ^= value;
        }
        value = aes_xtime(value);
        factor >>= 1;
    }
    out
}

pub(in crate::arch::arm64::execute) fn aes_xtime(value: u8) -> u8 {
    (value << 1) ^ if (value & 0x80) != 0 { 0x1B } else { 0 }
}
