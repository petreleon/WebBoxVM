//! AArch64 instruction decoder (pattern-based).
//!
//! Primary decoder: disarm64 (spec-driven, 3000+ instructions, 2x faster).
//! Fallback: our hand-rolled decoder for instructions disarm64 doesn't handle.

mod branch;
mod data_proc;
mod disarm64_shim;
mod ldst;
mod legacy;
mod system;

use super::opcodes::{Instr, Opcode};

/// Decode a raw 32-bit word into an instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    disarm64_shim::decode(raw)
}

pub(in crate::arm64::decode) enum DecodeStep {
    Hit(Instr),
    Reject,
    Miss,
}

impl DecodeStep {
    pub(in crate::arm64::decode) fn from_option(instr: Option<Instr>) -> Self {
        match instr {
            Some(instr) => Self::Hit(instr),
            None => Self::Reject,
        }
    }
}

/// Legacy hand-rolled decoder (fallback within the shim).
pub(crate) fn decode_legacy(raw: u32) -> Option<Instr> {
    legacy::decode(raw)
}

#[cfg(test)]
mod tests;
