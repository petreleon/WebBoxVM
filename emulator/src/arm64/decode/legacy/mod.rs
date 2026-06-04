mod early_sve_vector;
mod fp_scalar;
mod load_store_helpers;
mod scalar_ldst_branch;
mod simd_compare;
mod simd_convert;
mod simd_crypto_a;
mod simd_dup_convert;
mod simd_fp_by_element;
mod simd_fp_compare;
mod simd_fp_compare_zero;
mod simd_fp_unary;
mod simd_fp_unary_more;
mod simd_immediates_a;
mod simd_load_multi;
mod simd_move_scalar_fp;
mod simd_permute_logic;
mod simd_permute_start;
mod simd_reduce_ext;
mod simd_shift_insert;
mod simd_shift_left;
mod simd_shift_pairwise;
mod simd_shift_right;
mod simd_structure;
mod simd_tail_system;
mod simd_widen_helpers;
mod simd_widen_integer;
mod sve_addsub;
mod sve_byte_store;
mod sve_compare;
mod sve_contiguous_load;
mod sve_dup;
mod sve_fp;
mod sve_fp_arith_imm;
mod sve_fp_compare;
mod sve_fp_convert;
mod sve_fp_dup_imm;
mod sve_fp_fexpa;
mod sve_fp_ftmad;
mod sve_fp_unary;
mod sve_logical_imm;
mod sve_logical_pred;
mod sve_predicate_ld1r;
mod sve_shift_imm;
mod sve_word_load_store;

use super::{DecodeStep, Instr, Opcode, branch, data_proc, ldst, system};
use fp_scalar::decode_fp_scalar;
use load_store_helpers::*;
use simd_convert::*;
use simd_load_multi::*;
use simd_shift_insert::*;
use simd_shift_left::*;
use simd_shift_right::*;
use simd_structure::*;
use simd_widen_helpers::*;

macro_rules! try_stage {
    ($stage:expr) => {
        match $stage {
            DecodeStep::Hit(instr) => return Some(instr),
            DecodeStep::Reject => return None,
            DecodeStep::Miss => {}
        }
    };
}

pub(super) fn decode(raw: u32) -> Option<Instr> {
    try_stage!(early_sve_vector::decode(raw));
    try_stage!(sve_word_load_store::decode(raw));
    try_stage!(sve_contiguous_load::decode(raw));
    try_stage!(sve_byte_store::decode(raw));
    try_stage!(sve_compare::decode(raw));
    try_stage!(sve_fp_convert::decode(raw));
    try_stage!(sve_fp_unary::decode(raw));
    try_stage!(sve_fp_fexpa::decode(raw));
    try_stage!(sve_fp_ftmad::decode(raw));
    try_stage!(sve_fp_compare::decode(raw));
    try_stage!(sve_fp::decode(raw));
    try_stage!(sve_dup::decode(raw));
    try_stage!(sve_shift_imm::decode(raw));
    try_stage!(sve_addsub::decode(raw));
    try_stage!(sve_logical_imm::decode(raw));
    try_stage!(sve_logical_pred::decode(raw));
    try_stage!(sve_predicate_ld1r::decode(raw));
    try_stage!(simd_dup_convert::decode(raw));
    try_stage!(simd_fp_by_element::decode(raw));
    try_stage!(simd_fp_compare::decode(raw));
    try_stage!(simd_fp_compare_zero::decode(raw));
    try_stage!(simd_move_scalar_fp::decode(raw));
    try_stage!(simd_fp_unary_more::decode(raw));
    try_stage!(simd_fp_unary::decode(raw));
    try_stage!(simd_compare::decode(raw));
    try_stage!(simd_widen_integer::decode(raw));
    try_stage!(simd_permute_start::decode(raw));
    try_stage!(simd_crypto_a::decode(raw));
    try_stage!(simd_shift_pairwise::decode(raw));
    try_stage!(simd_reduce_ext::decode(raw));
    try_stage!(simd_permute_logic::decode(raw));
    try_stage!(simd_immediates_a::decode(raw));
    try_stage!(simd_tail_system::decode(raw));
    try_stage!(scalar_ldst_branch::decode(raw));
    None
}
