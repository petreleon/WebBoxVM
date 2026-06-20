//! shim: disarm64 vs legacy cross-validator.
//!
//! Decode with both disarm64 and our legacy decoder, cross-validate the
//! opcode, and return the legacy Instr (which has proven operand extraction).
//! This gives us disarm64 correctness validation without operand extraction bugs.

mod atomic_map;
mod atomic_mnemonics;
#[cfg(test)]
mod atomic_tests;
#[cfg(test)]
mod branch_tests;
#[cfg(test)]
mod cmp_tests;
mod core_map;
mod exclusive_map;
#[cfg(test)]
mod exclusive_tests;
mod fp_compare_map;
mod fp_map;
#[cfg(test)]
mod fp_scalar_tests;
mod helpers;
#[cfg(test)]
mod logical_alias_tests;
mod mops_map;
#[cfg(test)]
mod mops_tests;
mod mte_map;
#[cfg(test)]
mod mte_tests;
#[cfg(test)]
mod multiply_tests;
mod scalar_alias_map;
#[cfg(test)]
mod scalar_alias_tests;
mod scalar_bit_map;
mod scalar_cssc_map;
#[cfg(test)]
mod scalar_cssc_tests;
#[cfg(test)]
mod scalar_ldst_tests;
mod simd_crypto_map;
mod simd_ldst_map;
mod simd_map;
#[cfg(test)]
mod simd_mnemonic_tests;
#[cfg(test)]
mod simd_ucvtf_tests;
mod sme_map;
mod sve_addsub_map;
mod sve_byte_mem_map;
mod sve_dup_map;
mod sve_fp_convert_map;
#[cfg(test)]
mod sve_fp_convert_tests;
mod sve_fp_fexpa_map;
#[cfg(test)]
mod sve_fp_fexpa_tests;
mod sve_fp_immediate_map;
#[cfg(test)]
mod sve_fp_immediate_tests;
mod sve_fp_scale_map;
#[cfg(test)]
mod sve_fp_scale_tests;
mod sve_fp_trig_map;
#[cfg(test)]
mod sve_fp_trig_tests;
mod sve_fp_unary_map;
#[cfg(test)]
mod sve_fp_unary_tests;
mod sve_index_map;
mod sve_logical_map;
mod sve_permute_map;
mod sve_predicate_map;
#[cfg(test)]
mod sve_predicate_tests;
mod sve_reverse_map;
mod sve_shift_map;
mod sve_unpack_map;
#[cfg(test)]
mod sve_unpack_tests;
mod sve_xar_map;
#[cfg(test)]
mod sve_xar_tests;
mod system_map;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
mod validate;

use super::super::opcodes::{Instr, Opcode};
use disarm64::decoder;
use helpers::*;

/// Decode with legacy, optionally validate against disarm64.
pub fn decode(raw: u32) -> Option<Instr> {
    let legacy = super::decode_legacy(raw)?;
    validate::validate(raw, legacy)
}

fn mnemonic_to_opcode(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    scalar_alias_map::map(raw, m)
        .or_else(|| sve_addsub_map::map(raw, m))
        .or_else(|| sve_dup_map::map(raw, m))
        .or_else(|| sve_shift_map::map(raw, m))
        .or_else(|| sve_index_map::map(raw, m))
        .or_else(|| sve_xar_map::map(raw, m))
        .or_else(|| sve_unpack_map::map(raw, m))
        .or_else(|| sve_permute_map::map(raw, m))
        .or_else(|| sve_reverse_map::map(raw, m))
        .or_else(|| sve_logical_map::map(raw, m))
        .or_else(|| sve_predicate_map::map(raw, m))
        .or_else(|| sve_fp_convert_map::map(raw, m))
        .or_else(|| sve_fp_fexpa_map::map(raw, m))
        .or_else(|| sve_fp_scale_map::map(raw, m))
        .or_else(|| sve_fp_trig_map::map(raw, m))
        .or_else(|| sve_fp_immediate_map::map(raw, m))
        .or_else(|| sve_fp_unary_map::map(raw, m))
        .or_else(|| sme_map::map(raw, m))
        .or_else(|| sve_byte_mem_map::map(raw, m))
        .or_else(|| scalar_cssc_map::map(raw, m))
        .or_else(|| scalar_bit_map::map(raw, m))
        .or_else(|| core_map::map(raw, m))
        .or_else(|| atomic_map::map(raw, m))
        .or_else(|| exclusive_map::map(m))
        .or_else(|| simd_crypto_map::map(m))
        .or_else(|| simd_map::map(raw, m))
        .or_else(|| fp_compare_map::map(raw, m))
        .or_else(|| fp_map::map(raw, m))
        .or_else(|| simd_ldst_map::map(raw, m))
        .or_else(|| mops_map::map(raw, m))
        .or_else(|| mte_map::map(raw, m))
        .or_else(|| system_map::map(raw, m))
}
