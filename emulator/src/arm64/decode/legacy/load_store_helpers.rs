pub(super) fn is_ldst_pair(raw: u32) -> bool {
    const PATTERNS: &[(u32, u32)] = &[
        (0x2800_0000, 0x7FC0_0000),
        (0x2840_0000, 0x7FC0_0000),
        (0x2880_0000, 0x7EC0_0000),
        (0x28C0_0000, 0x7EC0_0000),
        (0x2900_0000, 0x7FC0_0000),
        (0x2940_0000, 0x7FC0_0000),
        (0x2C00_0000, 0x3FC0_0000),
        (0x2C40_0000, 0x3FC0_0000),
        (0x2C80_0000, 0x3EC0_0000),
        (0x2CC0_0000, 0x3EC0_0000),
        (0x2D00_0000, 0x3FC0_0000),
        (0x2D40_0000, 0x3FC0_0000),
        (0x68C0_0000, 0xFEC0_0000),
        (0x6940_0000, 0xFFC0_0000),
    ];

    PATTERNS
        .iter()
        .any(|(opcode, mask)| (raw & mask) == *opcode)
}
