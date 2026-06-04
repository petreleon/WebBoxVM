pub(in crate::arm64::execute) fn simd_replicate_byte(byte: u8) -> u128 {
    let mut value = 0u128;
    for lane in 0..16 {
        value |= (byte as u128) << (lane * 8);
    }
    value
}

pub(in crate::arm64::execute) fn simd_vector_mask(vector_size: usize) -> u128 {
    match vector_size {
        0 => 0,
        16.. => u128::MAX,
        bytes => (1u128 << (bytes * 8)) - 1,
    }
}

pub(in crate::arm64::execute) fn simd_element_mask(element_size: usize) -> u128 {
    let bits = element_size * 8;
    if bits >= 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    }
}

pub(in crate::arm64::execute) fn simd_byte(value: u128, lane: usize) -> u8 {
    ((value >> (lane * 8)) & 0xff) as u8
}

pub(in crate::arm64::execute) fn simd_element(
    value: u128,
    lane: usize,
    element_size: usize,
) -> u128 {
    let shift = lane * element_size * 8;
    (value >> shift) & simd_element_mask(element_size)
}

pub(in crate::arm64::execute) fn simd_reverse_elements_in_groups(
    value: u128,
    element_size: usize,
    vector_size: usize,
    group_size: usize,
) -> u128 {
    let elements_per_group = group_size / element_size;
    let groups = vector_size / group_size;
    let mut out = 0u128;
    for group in 0..groups {
        for index in 0..elements_per_group {
            let dst_lane = group * elements_per_group + index;
            let src_lane = group * elements_per_group + (elements_per_group - 1 - index);
            let element = simd_element(value, src_lane, element_size);
            out |= element << (dst_lane * element_size * 8);
        }
    }
    out & simd_vector_mask(vector_size)
}

pub(in crate::arm64::execute) fn simd_signed_element(
    value: u128,
    lane: usize,
    element_size: usize,
) -> i64 {
    simd_signed_element_value(simd_element(value, lane, element_size), element_size)
}

pub(in crate::arm64::execute) fn simd_signed_element_value(raw: u128, element_size: usize) -> i64 {
    let bits = element_size * 8;
    if bits == 64 {
        raw as u64 as i64
    } else {
        let sign = 1u128 << (bits - 1);
        let extended = if (raw & sign) != 0 {
            raw | (!0u128 << bits)
        } else {
            raw
        };
        extended as i128 as i64
    }
}

pub(in crate::arm64::execute) fn simd_replicate_element(
    element: u128,
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
    let mut value = 0u128;
    for lane in 0..lanes {
        value |= (element & element_mask) << (lane * bits);
    }
    value
}
