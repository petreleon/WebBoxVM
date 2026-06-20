pub(in crate::arch::arm64::execute) fn sve_pred_count(pattern: u8, elements: u64) -> u64 {
    match pattern {
        0 => highest_power_of_two_le(elements),
        1..=8 => exact_pred_count(pattern as u64, elements),
        9 => exact_pred_count(16, elements),
        10 => exact_pred_count(32, elements),
        11 => exact_pred_count(64, elements),
        12 => exact_pred_count(128, elements),
        13 => exact_pred_count(256, elements),
        29 => elements - elements % 4,
        30 => elements - elements % 3,
        31 => elements,
        literal => exact_pred_count(literal as u64, elements),
    }
}

pub(in crate::arch::arm64::execute) fn exact_pred_count(count: u64, elements: u64) -> u64 {
    if count <= elements { count } else { 0 }
}

pub(in crate::arch::arm64::execute) fn highest_power_of_two_le(value: u64) -> u64 {
    if value == 0 {
        0
    } else {
        1 << (u64::BITS - 1 - value.leading_zeros())
    }
}
