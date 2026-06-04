use super::*;

pub(super) fn vector_bytes(offset: u64) -> u128 {
    let mut value = 0u128;
    for lane in 0..16u64 {
        value |= ((lane + offset) as u128) << (lane * 8);
    }
    value
}

pub(super) fn ld_structure_vector_bytes(
    first_byte: u64,
    structure_count: u64,
    structure_index: u64,
    element_size: u64,
    lanes: u64,
) -> u128 {
    let mut value = 0u128;
    for lane in 0..lanes {
        for byte_index in 0..element_size {
            let byte =
                first_byte + (lane * structure_count + structure_index) * element_size + byte_index;
            value |= (byte as u128) << ((lane * element_size + byte_index) * 8);
        }
    }
    value
}

pub(super) fn f64_lane(cpu: &Armv8Cpu, reg: usize) -> f64 {
    f64::from_bits(cpu.simd[reg] as u64)
}

pub(super) fn f32_lane(cpu: &Armv8Cpu, reg: usize) -> f32 {
    f32::from_bits(cpu.simd[reg] as u32)
}

pub(super) fn f32x4(values: [f32; 4]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value.to_bits() as u128) << (lane * 32))
        })
}

pub(super) fn f64x2(values: [f64; 2]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value.to_bits() as u128) << (lane * 64))
        })
}

pub(super) fn u32x4(values: [u32; 4]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u128) << (lane * 32))
        })
}

pub(super) fn u64x2(values: [u64; 2]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u128) << (lane * 64))
        })
}

pub(super) fn i32x4(values: [i32; 4]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u32 as u128) << (lane * 32))
        })
}

pub(super) fn i64x2(values: [i64; 2]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u64 as u128) << (lane * 64))
        })
}

pub(super) fn u32_lane(value: u128, lane: usize) -> u32 {
    ((value >> (lane * 32)) & 0xffff_ffff) as u32
}

pub(super) fn u64_lane(value: u128, lane: usize) -> u64 {
    ((value >> (lane * 64)) & u64::MAX as u128) as u64
}

pub(super) fn polynomial_mult_u64(lhs: u64, rhs: u64) -> u128 {
    let mut out = 0u128;
    for bit in 0..64 {
        if ((rhs >> bit) & 1) != 0 {
            out ^= (lhs as u128) << bit;
        }
    }
    out
}

pub(super) fn sha256su0_expected(d: u128, n: u128) -> u128 {
    let schedule = [
        u32_lane(d, 1),
        u32_lane(d, 2),
        u32_lane(d, 3),
        u32_lane(n, 0),
    ];
    let mut out = [0u32; 4];
    for lane in 0..4 {
        let sigma0 = schedule[lane].rotate_right(7)
            ^ schedule[lane].rotate_right(18)
            ^ (schedule[lane] >> 3);
        out[lane] = u32_lane(d, lane).wrapping_add(sigma0);
    }
    u32x4(out)
}

pub(super) fn sha512su0_expected(d: u128, n: u128) -> u128 {
    let w0 = u64_lane(d, 0);
    let w1 = u64_lane(d, 1);
    let x0 = u64_lane(n, 0);
    let sig_w1 = w1.rotate_right(1) ^ w1.rotate_right(8) ^ (w1 >> 7);
    let sig_x0 = x0.rotate_right(1) ^ x0.rotate_right(8) ^ (x0 >> 7);
    u64x2([w0.wrapping_add(sig_w1), w1.wrapping_add(sig_x0)])
}

pub(super) fn sm3partw1_expected(d: u128, n: u128, m: u128) -> u128 {
    let mut words = [0u32; 4];
    for lane in 0..3 {
        words[lane] =
            (u32_lane(d, lane) ^ u32_lane(n, lane)) ^ u32_lane(m, lane + 1).rotate_left(15);
    }
    for lane in 0..4 {
        if lane == 3 {
            words[3] = (u32_lane(d, 3) ^ u32_lane(n, 3)) ^ words[0].rotate_left(15);
        }
        words[lane] ^= words[lane].rotate_left(15) ^ words[lane].rotate_left(23);
    }
    u32x4(words)
}
