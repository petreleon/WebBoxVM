use super::*;

pub(in crate::arm64::execute) fn simd_pairwise_binary<F>(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    f: F,
) -> u128
where
    F: Fn(u128, u128, u128) -> u128,
{
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let elements = vector_size / element_size;
    let pairs_per_source = elements / 2;
    let mut out = 0u128;
    for lane in 0..elements {
        let source = if lane < pairs_per_source { lhs } else { rhs };
        let pair = (lane % pairs_per_source) * 2;
        let a = simd_element(source, pair, element_size);
        let b = simd_element(source, pair + 1, element_size);
        out |= (f(a, b, element_mask) & element_mask) << (lane * bits);
    }
    out
}

pub(in crate::arm64::execute) fn simd_zip(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    high_half: bool,
) -> u128 {
    let bits = element_size * 8;
    let lanes = vector_size / element_size;
    let half = lanes / 2;
    let start = if high_half { half } else { 0 };
    let mut out = 0u128;
    for lane in 0..half {
        out |= simd_element(lhs, start + lane, element_size) << ((lane * 2) * bits);
        out |= simd_element(rhs, start + lane, element_size) << ((lane * 2 + 1) * bits);
    }
    out & simd_vector_mask(vector_size)
}

pub(in crate::arm64::execute) fn simd_trn(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    high_half: bool,
) -> u128 {
    let bits = element_size * 8;
    let lanes = vector_size / element_size;
    let start = if high_half { 1 } else { 0 };
    let mut out = 0u128;
    for pair in 0..(lanes / 2) {
        let source_lane = start + pair * 2;
        out |= simd_element(lhs, source_lane, element_size) << ((pair * 2) * bits);
        out |= simd_element(rhs, source_lane, element_size) << ((pair * 2 + 1) * bits);
    }
    out & simd_vector_mask(vector_size)
}

pub(in crate::arm64::execute) fn simd_compare_elements_with_zero(
    value: u128,
    element_size: usize,
    vector_size: usize,
) -> u128 {
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        if simd_element(value, lane, element_size) == 0 {
            out |= element_mask << (lane * bits);
        }
    }
    out
}

// ── Register extension & shifting ──
