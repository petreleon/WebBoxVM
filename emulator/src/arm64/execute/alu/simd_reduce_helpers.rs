use super::*;

pub(in crate::arm64::execute) fn signed_max(a: u128, b: u128, element_size: usize) -> u128 {
    if simd_signed_element_value(a, element_size) >= simd_signed_element_value(b, element_size) {
        a
    } else {
        b
    }
}

pub(in crate::arm64::execute) fn signed_min(a: u128, b: u128, element_size: usize) -> u128 {
    if simd_signed_element_value(a, element_size) <= simd_signed_element_value(b, element_size) {
        a
    } else {
        b
    }
}

pub(in crate::arm64::execute) fn unsigned_reduce(
    value: u128,
    element_size: usize,
    vector_size: usize,
    max: bool,
) -> u128 {
    let lanes = vector_size / element_size;
    let mut out = simd_element(value, 0, element_size);
    for lane in 1..lanes {
        let next = simd_element(value, lane, element_size);
        out = if max { out.max(next) } else { out.min(next) };
    }
    out
}

pub(in crate::arm64::execute) fn signed_reduce(
    value: u128,
    element_size: usize,
    vector_size: usize,
    max: bool,
) -> u128 {
    let lanes = vector_size / element_size;
    let mut out = simd_element(value, 0, element_size);
    for lane in 1..lanes {
        let next = simd_element(value, lane, element_size);
        out = if max {
            signed_max(out, next, element_size)
        } else {
            signed_min(out, next, element_size)
        };
    }
    out
}
