//! Decode AArch64 logical bitmask immediates (AND, ORR, EOR, ANDS).
//!
//! ARM64 can encode many 64-bit patterns in just 13 bits using element
//! replication: a small bit-pattern is rotated and repeated to fill the
//! full register (32 or 64 bits).
//!
//! Reference: ARM Architecture Reference Manual, "DecodeBitMasks"

/// Decode a 13-bit bitmask encoding into a 64-bit immediate value.
///
/// Returns `None` if the encoding is reserved/invalid.
pub fn decode_bitmask_imm(n: u32, immr: u32, imms: u32, is_64bit: bool) -> Option<u64> {
    // 32-bit mode requires N = 0
    if !is_64bit && n != 0 {
        return None;
    }

    // ARM reserved patterns: all-ones in the element (would mean "select everything")
    if is_64bit && n == 1 && imms == 0b111111 {
        return None;
    }
    if !is_64bit && n == 0 && imms == 0b011111 {
        return None;
    }

    // Step 1: Compute the element size (len = position of highest set bit in
    // the concatenation of N and the bitwise-NOT of imms).
    let combined: u32 = (n << 6) | ((!imms) & 0x3f);
    let len = 31i32 - (combined.leading_zeros() as i32);
    if len < 0 {
        return None;
    }

    let mut size = 1u32 << len;

    // Step 2: Extract rotation (R) and ones-count (S) within the element.
    let r = immr & (size - 1);
    let s = imms & (size - 1);

    // Step 3: Build the base pattern — (S+1) consecutive ones.
    let mut pattern = (1u64 << (s + 1)) - 1;

    // Step 4: Rotate right by R within the size-bit field.
    pattern = rotate_right_within(pattern, r, size);

    // Step 5: Replicate the element to fill the full register.
    let reg_size = if is_64bit { 64 } else { 32 };
    while size != reg_size {
        pattern |= pattern << size;
        size *= 2;
    }

    Some(pattern)
}

/// Rotate a value right by `rot` bits within a `size`-bit field.
fn rotate_right_within(val: u64, rot: u32, size: u32) -> u64 {
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
    fn bitmask_1() {
        assert_eq!(decode_bitmask_imm(1, 0, 0, true).unwrap(), 0x1);
    }
    #[test]
    fn bitmask_3() {
        assert_eq!(decode_bitmask_imm(1, 0, 1, true).unwrap(), 0x3);
    }
    #[test]
    fn bitmask_7() {
        assert_eq!(decode_bitmask_imm(1, 0, 2, true).unwrap(), 0x7);
    }
    #[test]
    fn bitmask_f() {
        assert_eq!(decode_bitmask_imm(1, 0, 3, true).unwrap(), 0xF);
    }
    #[test]
    fn bitmask_ff() {
        assert_eq!(decode_bitmask_imm(1, 0, 7, true).unwrap(), 0xFF);
    }
    #[test]
    fn bitmask_32bit_replicated_1() {
        let v = decode_bitmask_imm(0, 0, 0, true).unwrap();
        assert_eq!(v, 0x0000000100000001);
    }
    #[test]
    fn bitmask_rotated_msb() {
        let v = decode_bitmask_imm(0, 1, 0, true).unwrap();
        assert_eq!(v, 0x8000000080000000);
    }
    #[test]
    fn bitmask_32bit_1() {
        assert_eq!(decode_bitmask_imm(0, 0, 0, false).unwrap(), 0x1);
    }
    #[test]
    fn bitmask_32bit_ff() {
        assert_eq!(decode_bitmask_imm(0, 0, 7, false).unwrap(), 0xFF);
    }
    #[test]
    fn reserved_all_ones() {
        assert!(decode_bitmask_imm(1, 0, 0x3F, true).is_none());
    }
    #[test]
    fn invalid_32bit_with_n1() {
        assert!(decode_bitmask_imm(1, 0, 0, false).is_none());
    }
}
