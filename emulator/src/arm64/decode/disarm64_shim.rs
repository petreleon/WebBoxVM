//! shim: disarm64 vs legacy cross-validator.
//!
//! Decode with both disarm64 and our legacy decoder, cross-validate the
//! opcode, and return the legacy Instr (which has proven operand extraction).
//! This gives us disarm64 correctness validation without operand extraction bugs.

mod atomic_map;
mod atomic_mnemonics;
#[cfg(test)]
mod atomic_tests;
mod core_map;
mod exclusive_map;
#[cfg(test)]
mod exclusive_tests;
mod fp_map;
mod helpers;
mod simd_ldst_map;
mod simd_map;
mod system_map;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;

use super::super::opcodes::{Instr, Opcode};
use disarm64::decoder;
use helpers::*;
#[cfg(debug_assertions)]
use std::sync::OnceLock;

/// Decode with legacy, validate against disarm64 (debug builds only).
pub fn decode(raw: u32) -> Option<Instr> {
    let legacy = super::decode_legacy(raw)?;

    // Cross-validate against disarm64 in debug builds
    #[cfg(debug_assertions)]
    if log_disarm64_mismatches() {
        if let Some(d64) = decoder::decode(raw) {
            if let Some(expected) = mnemonic_to_opcode(raw, d64.mnemonic) {
                if legacy.op != expected {
                    eprintln!(
                        "DISARM64 MISMATCH: raw=0x{raw:08x} legacy={:?} disarm64={:?}",
                        legacy.op, expected
                    );
                }
            }
        }
    }

    Some(legacy)
}

#[cfg(debug_assertions)]
fn log_disarm64_mismatches() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WEBBOXVM_DISARM64_MISMATCHES").is_some())
}

fn mnemonic_to_opcode(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    core_map::map(raw, m)
        .or_else(|| atomic_map::map(raw, m))
        .or_else(|| exclusive_map::map(m))
        .or_else(|| fp_map::map(raw, m))
        .or_else(|| simd_map::map(raw, m))
        .or_else(|| simd_ldst_map::map(raw, m))
        .or_else(|| system_map::map(raw, m))
}
