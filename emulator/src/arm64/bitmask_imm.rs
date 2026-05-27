//! Decode AArch64 logical bitmask immediates (AND, ORR, EOR, TST).

/// Decode a 13-bit encoding (N:immr:imms) into a 64-bit immediate value.
/// Returns None if the encoding is invalid/reserved.
///
/// Algorithm from ARM Architecture Reference Manual (matches LLVM AArch64 backend):
/// 1. len = highest set bit of (N concatenated with bitwise-NOT of imms)
/// 2. size = 2^len
/// 3. S = imms & (size-1), R = immr & (size-1)
/// 4. If S == size-1, encoding is reserved
/// 5. pattern = (1 << (S+1)) - 1  // S+1 consecutive ones
/// 6. Rotate pattern right by R bits within size-bit field
/// 7. Replicate pattern to fill 32 or 64 bits
pub fn decode_bitmask_imm(n: u32, immr: u32, imms: u32, is_64bit: bool) -> Option<u64> {
    if !is_64bit && n != 0 {
        return None;
    }

    // ARM reserved patterns: N:imms == '1 111111' (64-bit) or '0 011111' (32-bit)
    if is_64bit && n == 1 && imms == 0b111111 {
        return None;
    }
    if !is_64bit && n == 0 && imms == 0b011111 {
        return None;
    }

    // Step 1: compute len
    let combined: u32 = (n << 6) | ((!imms) & 0x3f);
    let len = 31i32 - (combined.leading_zeros() as i32);
    if len < 0 {
        return None;
    }

    // Step 2: element size
    let mut size = 1u32 << len;

    // Step 3: extract S and R
    let r = immr & (size - 1);
    let s = imms & (size - 1);

    // Step 5: base pattern
    let mut pattern = (1u64 << (s + 1)) - 1;

    // Step 6: rotate right by R bits
    pattern = ror64(pattern, r, size);

    // Step 7: replicate to fill register
    let reg_size = if is_64bit { 64 } else { 32 };
    while size != reg_size {
        pattern |= pattern << size;
        size *= 2;
    }

    Some(pattern)
}

/// Right-rotate a value within a size-bit field.
fn ror64(val: u64, rot: u32, size: u32) -> u64 {
    if rot == 0 || size == 0 {
        return val;
    }
    let mask = if size == 64 { u64::MAX } else { (1u64 << size) - 1 };
    let v = val & mask;
    ((v >> rot) | (v << (size - rot))) & mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_bitmask_0x1_64bit() {
        let v = decode_bitmask_imm(1, 0, 0, true).unwrap();
        assert_eq!(v, 0x1);
    }

    #[test]
    fn decode_bitmask_0x3_64bit() {
        let v = decode_bitmask_imm(1, 0, 1, true).unwrap();
        assert_eq!(v, 0x3);
    }

    #[test]
    fn decode_bitmask_0x7_64bit() {
        let v = decode_bitmask_imm(1, 0, 2, true).unwrap();
        assert_eq!(v, 0x7);
    }

    #[test]
    fn decode_bitmask_0xF_64bit() {
        let v = decode_bitmask_imm(1, 0, 3, true).unwrap();
        assert_eq!(v, 0xF);
    }

    #[test]
    fn decode_bitmask_0xFF_64bit() {
        let v = decode_bitmask_imm(1, 0, 7, true).unwrap();
        assert_eq!(v, 0xFF);
    }

    #[test]
    fn decode_bitmask_0x5555_replicated() {
        // 32-bit pattern 0x1 replicated across 64 bits
        let v = decode_bitmask_imm(0, 0, 0, true).unwrap();
        assert_eq!(v, 0x0000000100000001);
    }

    #[test]
    fn decode_bitmask_0x5555_rotated() {
        // 32-bit pattern 0x1 rotated right by 1 = 0x80000000, replicated
        let v = decode_bitmask_imm(0, 1, 0, true).unwrap();
        assert_eq!(v, 0x8000000080000000);
    }

    #[test]
    fn decode_bitmask_0x3333_replicated() {
        // 32-bit pattern 0x3 replicated
        let v = decode_bitmask_imm(0, 0, 1, true).unwrap();
        assert_eq!(v, 0x0000000300000003);
    }

    #[test]
    fn decode_bitmask_0x7777_replicated() {
        // 32-bit pattern 0x7 replicated
        let v = decode_bitmask_imm(0, 0, 2, true).unwrap();
        assert_eq!(v, 0x0000000700000007);
    }

    #[test]
    fn decode_bitmask_32bit_0x1() {
        let v = decode_bitmask_imm(0, 0, 0, false).unwrap();
        assert_eq!(v, 0x1);
    }

    #[test]
    fn decode_bitmask_32bit_0xFF() {
        let v = decode_bitmask_imm(0, 0, 7, false).unwrap();
        assert_eq!(v, 0xFF);
    }

    #[test]
    fn decode_bitmask_reserved_all_ones() {
        // N=1, imms=0x3F => S=63, size=64, S==size-1 => reserved
        assert!(decode_bitmask_imm(1, 0, 0x3F, true).is_none());
    }

    #[test]
    fn decode_bitmask_invalid_32bit_with_N1() {
        assert!(decode_bitmask_imm(1, 0, 0, false).is_none());
    }

    #[test]
    fn decode_bitmask_0x10101010() {
        // Find encoding for 0x10101010... pattern
        // 32-bit pattern 0x01, replicated across 64 bits
        let v = decode_bitmask_imm(0, 0, 0, true).unwrap();
        assert_eq!(v, 0x0000000100000001);
    }
}
