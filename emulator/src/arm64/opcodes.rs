//! Instruction opcodes and decoded representation.

mod core_variants;
mod fp_variants;
mod simd_variants;
mod sve_variants;
mod system_variants;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instr {
    pub op: Opcode,
    pub rd: u8,
    pub rn: u8,
    pub rm: u8,
    pub imm: u64,
    pub sf: bool,
    pub cond: u8,
    pub size: u8, // access size in bytes for LDR/STR (0=unused)
}

impl Instr {
    pub const fn nop() -> Self {
        Self {
            op: Opcode::Nop,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 0,
        }
    }
}
