pub(in crate::arm64::execute) fn fp_expand_imm(imm8: u8, size: u8) -> u64 {
    let sign = (imm8 >> 7) as u64;
    let b = ((imm8 >> 6) & 1) as u64;
    let c = ((imm8 >> 5) & 1) as u64;
    let d = ((imm8 >> 4) & 1) as u64;
    let fraction = (imm8 & 0xF) as u64;

    match size {
        2 => {
            let exponent = ((!b & 1) << 4) | ((if b == 1 { 0x3 } else { 0 }) << 2) | (c << 1) | d;
            (sign << 15) | (exponent << 10) | (fraction << 6)
        }
        4 => {
            let exponent = ((!b & 1) << 7) | ((if b == 1 { 0x1F } else { 0 }) << 2) | (c << 1) | d;
            (sign << 31) | (exponent << 23) | (fraction << 19)
        }
        _ => {
            let exponent = ((!b & 1) << 10) | ((if b == 1 { 0xFF } else { 0 }) << 2) | (c << 1) | d;
            (sign << 63) | (exponent << 52) | (fraction << 48)
        }
    }
}
