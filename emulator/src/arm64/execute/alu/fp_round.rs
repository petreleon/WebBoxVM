pub(in crate::arm64::execute) fn round_fpcr(value: f64, fpcr: u64) -> f64 {
    match (fpcr >> 22) & 0b11 {
        0b00 => round_ties_even(value),
        0b01 => value.ceil(),
        0b10 => value.floor(),
        _ => value.trunc(),
    }
}

pub(in crate::arm64::execute) fn round_ties_even(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }

    let trunc = value.trunc();
    let frac = (value - trunc).abs();
    if frac < 0.5 {
        trunc
    } else if frac > 0.5 {
        trunc + value.signum()
    } else if (trunc.abs() % 2.0) == 0.0 {
        trunc
    } else {
        trunc + value.signum()
    }
}
