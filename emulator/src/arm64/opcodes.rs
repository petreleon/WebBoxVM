//! Instruction opcodes and decoded representation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Movz,
    Ldr,
    LdrLit,
    Str,
    B,
    Br,
    Nop,
    Bl,
    Blr,
    Ret,
    Cbz,
    Cbnz,
    BCond,
    Ldp,
    Stp,
    MovReg,
    AddImm,
    SubImm,
    CmpImm,
    Cmp,
    Adrp,
    Adr,
    Tbz,
    Tbnz,
    Movk,
    Movn,
    Sxtw,
    Csel,
    Ccmp,
    NopBarrier, // DSB, ISB, DMB
}

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
