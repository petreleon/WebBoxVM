use super::*;
use crate::arm64::decode::decode;
use crate::constants::{
    DESC_ADDR_MASK, DESC_AF_BIT, DESC_AP_EL0, DESC_TABLE, DESC_VALID, ESR_EC_SVC64, ESR_EC_UNKNOWN,
    ESR_IL, PAGE_SIZE, PHYSICAL_TIMER_IRQ_ID, PSTATE_DAIF_MASK, PSTATE_EL_MASK, PSTATE_I_BIT,
    PT_L1_SHIFT, PT_L2_SHIFT, PT_L3_SHIFT, RAM_BASE, SCTLR_MMU_ENABLE, SYSREG_CNTKCTL_EL1,
    SYSREG_CNTP_TVAL_EL0, SYSREG_CNTV_CTL_EL0, SYSREG_CNTV_TVAL_EL0, TCR_T1SZ_SHIFT,
    TIMER_CTL_ENABLE, TIMER_CTL_IMASK, VBAR_IRQ_CURRENT_EL, VBAR_IRQ_LOWER_EL_AARCH64,
    VBAR_SYNC_LOWER_EL_AARCH64, VIRTUAL_TIMER_IRQ_ID,
};

fn setup() -> (Armv8Cpu, SystemBus) {
    (Armv8Cpu::new(), SystemBus::new())
}

fn pred_bit(cpu: &Armv8Cpu, pred: usize, bit: usize) -> bool {
    (cpu.sve_pred[pred][bit / 64] & (1 << (bit % 64))) != 0
}

fn z_elem(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u64 {
    let offset = lane * 8;
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&cpu.sve_z[reg][offset..offset + 8]);
    u64::from_le_bytes(bytes)
}

fn z_word(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u32 {
    let offset = lane * 4;
    let mut bytes = [0; 4];
    bytes.copy_from_slice(&cpu.sve_z[reg][offset..offset + 4]);
    u32::from_le_bytes(bytes)
}

fn set_z_elem(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u64) {
    let offset = lane * 8;
    cpu.sve_z[reg][offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn sync_simd_alias(cpu: &mut Armv8Cpu, reg: usize) {
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&cpu.sve_z[reg][..16]);
    cpu.simd[reg] = u128::from_le_bytes(bytes);
}

fn map_two_user_pages(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va_page: u64,
    first_pa: u64,
    second_pa: u64,
) {
    let l1 = RAM_BASE;
    let l2 = RAM_BASE + PAGE_SIZE;
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let l1_idx = (va_page >> PT_L1_SHIFT) & 0x1ff;
    let l2_idx = (va_page >> PT_L2_SHIFT) & 0x1ff;
    let l3_idx = (va_page >> PT_L3_SHIFT) & 0x1ff;
    let table_desc = |pa: u64| (pa & DESC_ADDR_MASK) | DESC_TABLE;
    let page_desc = |pa: u64| (pa & DESC_ADDR_MASK) | DESC_VALID | DESC_AF_BIT | DESC_AP_EL0;

    bus.mem.write(l1 + l1_idx * 8, 8, table_desc(l2));
    bus.mem.write(l2 + l2_idx * 8, 8, table_desc(l3));
    bus.mem.write(l3 + l3_idx * 8, 8, page_desc(first_pa));
    bus.mem
        .write(l3 + (l3_idx + 1) * 8, 8, page_desc(second_pa));

    cpu.sys.ttbr0_el1 = l1;
    cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
}

mod atomics;
mod basic_alu;
mod branch;
mod data_proc;
mod exceptions;
mod load_store;
mod mops;
mod mte;
mod scalar_addsub_ext;
mod scalar_bitfield;
mod scalar_fp_arithmetic;
mod scalar_fp_compare;
mod scalar_fp_conversion;
mod scalar_fp_fused;
mod scalar_fp_minmax;
mod scalar_variable_shift;
mod simd_arithmetic;
mod simd_basic;
mod simd_cmhs;
mod simd_cmp_sat_strlen;
mod simd_compare_zero;
mod simd_crypto;
mod simd_fp_compare;
mod simd_fp_minmax;
mod simd_fp_mulx;
mod simd_fp_pairwise;
mod simd_fp_unary_more;
mod simd_fp_vector;
mod simd_helpers;
mod simd_ld1_multi;
mod simd_ld1r;
mod simd_minmax;
mod simd_narrow_high2;
mod simd_narrow_round;
mod simd_permute_secondary;
mod simd_reduce_across;
mod simd_scalar_bitwise_compare;
mod simd_scalar_load_store;
mod simd_sha1;
mod simd_sha256;
mod simd_sha512;
mod simd_shift_insert;
mod simd_signed_compare;
mod simd_sm3;
mod simd_struct_store;
mod simd_table_permute;
mod simd_userland_permute_reduction;
mod simd_widen_addwide;
mod simd_widen_mul;
mod simd_word_immediate;
mod simd_xtn2;
mod sme_memory;
mod sve_addsub;
mod sve_byte_load_store;
mod sve_compare;
mod sve_counts;
mod sve_dup;
mod sve_fp_compare;
mod sve_fp_convert;
mod sve_fp_div;
mod sve_fp_fexpa;
mod sve_fp_immediate;
mod sve_fp_scale;
mod sve_fp_trig;
mod sve_fp_unary;
mod sve_fp_unpredicated;
mod sve_logical_imm;
mod sve_logical_pred;
mod sve_permute;
mod sve_predicate;
mod sve_predicated_dword_load_store;
mod sve_register_load_store;
mod sve_shift_imm;
mod sve_unpack;
mod sve_whilelo;
mod sve_word_load_store;
mod sve_xar;
mod sve_z_vector;
mod system_extensions;
mod system_misc;
mod timers;
