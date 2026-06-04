use super::*;

mod adrp;
mod atomics;
mod basic_data;
mod branches_conditions;
mod busybox_crypto_fields;
mod busybox_fp_cases;
mod busybox_fp_fields;
mod busybox_simd_cases;
mod exclusive;
mod loads_literals;
mod logical_aliases;
mod multiply_long;
mod pairs_and_initial_simd;
mod scalar_aliases;
mod scalar_ldst;
mod simd_pairwise_narrow;
mod simd_userland_arith_move;
mod simd_userland_dup;
mod simd_userland_fp_imm_cmp;
mod simd_userland_ld1;
mod simd_userland_logical;
mod simd_userland_struct_ldst;
mod simd_userland_widen_fp;
mod sve_counts;
mod sve_load_store;
mod sve_predicate;
mod sve_z_vector;
mod system_misc;

fn assert_decode_cases(cases: &[(u32, Opcode, &str)]) {
    for &(raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }
}

fn assert_disarm64_mnemonic(raw: u32, mnemonic: &str) {
    let decoded = disarm64::decoder::decode(raw).expect("disarm64 should decode test word");
    assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
}
