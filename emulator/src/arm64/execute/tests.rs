use super::*;
use crate::arm64::decode::decode;
use crate::constants::{
    DESC_ADDR_MASK, DESC_AF_BIT, DESC_AP_EL0, DESC_TABLE, DESC_VALID, ESR_EC_SVC64, PAGE_SIZE,
    PHYSICAL_TIMER_IRQ_ID, PSTATE_DAIF_MASK, PSTATE_EL_MASK, PSTATE_I_BIT, PT_L1_SHIFT,
    PT_L2_SHIFT, PT_L3_SHIFT, RAM_BASE, SCTLR_MMU_ENABLE, SYSREG_CNTKCTL_EL1, SYSREG_CNTP_TVAL_EL0,
    SYSREG_CNTV_CTL_EL0, SYSREG_CNTV_TVAL_EL0, TCR_T1SZ_SHIFT, TIMER_CTL_ENABLE, TIMER_CTL_IMASK,
    VBAR_IRQ_CURRENT_EL, VBAR_IRQ_LOWER_EL_AARCH64, VBAR_SYNC_LOWER_EL_AARCH64,
    VIRTUAL_TIMER_IRQ_ID,
};

fn setup() -> (Armv8Cpu, SystemBus) {
    (Armv8Cpu::new(), SystemBus::new())
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

#[test]
fn add_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 10);
    cpu.regs.set_x(2, 32);
    execute(&mut cpu, &mut bus, decode(0x8B02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn sub_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 50);
    cpu.regs.set_x(2, 8);
    execute(&mut cpu, &mut bus, decode(0xCB02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn sbc_xzr_xzr_builds_unsigned_borrow_mask() {
    let (mut cpu, mut bus) = setup();

    cpu.pstate.set_nzcv(true, false, false, false);
    execute(&mut cpu, &mut bus, decode(0xDA1F_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), u64::MAX);

    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0xDA1F_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0);
}

#[test]
fn nop_advances_pc() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

#[test]
fn simd_bic_immediate_clears_replicated_halfword_mask() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[4] = 0x1234_f0ff_abcd_00f0_ffff_0f0f_55aa_aa55;

    execute(&mut cpu, &mut bus, decode(0x6F07_9604).unwrap()).unwrap();

    assert_eq!(cpu.simd[4], 0x1204_f00f_ab0d_0000_ff0f_0f0f_550a_aa05);
}

#[test]
fn simd_umov_zero_extends_halfword_to_w_register() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[30] = 0x7777_6666_5555_4444_3333_2222_1111_abcd;
    cpu.regs.set_x(0, u64::MAX);

    execute(&mut cpu, &mut bus, decode(0x0E02_3FC0).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0xabcd);
}

#[test]
fn simd_ext_extracts_concatenated_bytes() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[1] = 0x0f0e_0d0c_0b0a_0908_0706_0504_0302_0100;
    cpu.simd[2] = 0x1f1e_1d1c_1b1a_1918_1716_1514_1312_1110;

    execute(&mut cpu, &mut bus, decode(0x6E02_4020).unwrap()).unwrap(); // ext v0.16b, v1.16b, v2.16b, #8

    assert_eq!(cpu.simd[0], 0x1716_1514_1312_1110_0f0e_0d0c_0b0a_0908);
}

#[test]
fn simd_pairwise_min_and_add_bytes() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[1] = 0x100f_0e0d_0c0b_0a09_0807_0605_0403_0201;
    cpu.simd[2] = 0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10;

    execute(&mut cpu, &mut bus, decode(0x6E22_AC20).unwrap()).unwrap(); // uminp v0.16b, v1.16b, v2.16b
    assert_eq!(cpu.simd[0], 0x0103_0507_090b_0d0f_0f0d_0b09_0705_0301);

    execute(&mut cpu, &mut bus, decode(0x4E22_BC45).unwrap()).unwrap(); // addp v5.16b, v2.16b, v2.16b
    assert_eq!(cpu.simd[5], 0x0307_0b0f_1317_1b1f_0307_0b0f_1317_1b1f);
}

#[test]
fn simd_userland_vector_permute_and_reduction_ops() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(20, 0xaabb_ccdd);
    cpu.regs.set_w(2, 0x1122_3344);

    execute(&mut cpu, &mut bus, decode(0x0E04_0E8F).unwrap()).unwrap(); // dup v15.2s, w20
    assert_eq!(cpu.simd[15], 0xaabb_ccdd_aabb_ccdd);

    execute(&mut cpu, &mut bus, decode(0x4E04_0C40).unwrap()).unwrap(); // dup v0.4s, w2
    assert_eq!(cpu.simd[0], 0x1122_3344_1122_3344_1122_3344_1122_3344);

    cpu.simd[24] = 0x00ff_00ff_00ff_00ff_1111_2222_3333_4444;
    cpu.simd[25] = 0xff00_ff00_ff00_ff00_8888_4444_2222_1111;
    execute(&mut cpu, &mut bus, decode(0x4EB9_1F18).unwrap()).unwrap(); // orr v24.16b, v24.16b, v25.16b
    assert_eq!(cpu.simd[24], 0xffff_ffff_ffff_ffff_9999_6666_3333_5555);

    cpu.simd[1] = 0xffff_0000_ffff_0000_1234_5678_9abc_def0;
    cpu.simd[0] = 0x0f0f_0f0f_f0f0_f0f0_ffff_0000_ffff_0000;
    execute(&mut cpu, &mut bus, decode(0x4E20_1C21).unwrap()).unwrap(); // and v1.16b, v1.16b, v0.16b
    assert_eq!(cpu.simd[1], 0x0f0f_0000_f0f0_0000_1234_0000_9abc_0000);

    cpu.simd[31] = 0x0102_0304_0506_0708_7f80_55aa_0001_0fff;
    execute(&mut cpu, &mut bus, decode(0x0E20_5BFF).unwrap()).unwrap(); // cnt v31.8b, v31.8b
    execute(&mut cpu, &mut bus, decode(0x0E31_BBFF).unwrap()).unwrap(); // addv b31, v31.8b
    assert_eq!(cpu.simd[31], 29);
}

#[test]
fn simd_ld1_and_st1_multi_load_consecutive_vectors() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x1000;
    cpu.regs.set_x(1, base);
    for byte in 0..32u64 {
        bus.write(base + byte, 1, byte);
    }

    execute(&mut cpu, &mut bus, decode(0x4C40_A03E).unwrap()).unwrap(); // ld1 {v30.16b, v31.16b}, [x1]

    assert_eq!(cpu.simd[30], vector_bytes(0));
    assert_eq!(cpu.simd[31], vector_bytes(16));

    let post_index_base = RAM_BASE + 0x1800;
    cpu.regs.set_x(16, post_index_base);
    for byte in 0..16u64 {
        bus.write(post_index_base + byte, 1, 0xa0 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CDF_7A04).unwrap()).unwrap(); // ld1 {v4.16b}, [x16], #16
    assert_eq!(cpu.simd[4], 0xafaeadac_abaaa9a8_a7a6a5a4_a3a2a1a0);
    assert_eq!(cpu.regs.x(16), post_index_base + 16);

    let out = RAM_BASE + 0x2000;
    cpu.regs.set_x(22, out);
    cpu.simd[30] = 0x0f0e_0d0c_0b0a_0908_0706_0504_0302_0100;
    cpu.simd[31] = 0x1f1e_1d1c_1b1a_1918_1716_1514_1312_1110;

    execute(&mut cpu, &mut bus, decode(0x4C00_A2DE).unwrap()).unwrap(); // st1 {v30.16b, v31.16b}, [x22]

    for byte in 0..32u64 {
        assert_eq!(bus.read(out + byte, 1), Some(byte));
    }

    let post_index_out = RAM_BASE + 0x2400;
    cpu.regs.set_x(17, post_index_out);
    cpu.simd[4] = 0x8f8e_8d8c_8b8a_8988_8786_8584_8382_8180;
    cpu.simd[5] = 0x9f9e_9d9c_9b9a_9998_9796_9594_9392_9190;

    execute(&mut cpu, &mut bus, decode(0x4C9F_AA24).unwrap()).unwrap(); // st1 {v4.16b, v5.16b}, [x17], #32

    for byte in 0..32u64 {
        assert_eq!(bus.read(post_index_out + byte, 1), Some(0x80 + byte));
    }
    assert_eq!(cpu.regs.x(17), post_index_out + 32);

    let st4_single_out = RAM_BASE + 0x2800;
    cpu.regs.set_x(17, st4_single_out);
    cpu.simd[4] = vector_bytes(0x40);

    execute(&mut cpu, &mut bus, decode(0x4C9F_7A24).unwrap()).unwrap(); // st4 {v4.4s}, [x17], #16

    for byte in 0..16u64 {
        assert_eq!(bus.read(st4_single_out + byte, 1), Some(0x40 + byte));
    }
    assert_eq!(cpu.regs.x(17), st4_single_out + 16);
}

#[test]
fn simd_ld1r_replicates_loaded_doubleword() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x3000;
    cpu.regs.set_x(0, base);
    bus.write(base, 8, 0x1122_3344_5566_7788);

    execute(&mut cpu, &mut bus, decode(0x4D40_CC1F).unwrap()).unwrap(); // ld1r {v31.2d}, [x0]

    assert_eq!(cpu.simd[31], 0x1122_3344_5566_7788_1122_3344_5566_7788);
}

#[test]
fn simd_userland_arithmetic_shift_and_insert_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_w(4, 0x3f80_0000);
    execute(&mut cpu, &mut bus, decode(0x1E27_009F).unwrap()).unwrap(); // fmov s31, w4
    assert_eq!(cpu.simd[31], 0x3f80_0000);

    cpu.simd[31] = 0xffff_ffff_8000_0001;
    cpu.regs.set_x(1, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0x1E26_03E1).unwrap()).unwrap(); // fmov w1, s31
    assert_eq!(cpu.regs.x(1), 0x8000_0001);

    cpu.simd[0] = 0x1111_1111_2222_2222;
    cpu.regs.set_x(3, 0xaaaa_bbbb_cccc_dddd);
    execute(&mut cpu, &mut bus, decode(0x9EAF_0060).unwrap()).unwrap(); // fmov v0.d[1], x3
    assert_eq!(cpu.simd[0], 0xaaaa_bbbb_cccc_dddd_1111_1111_2222_2222);

    cpu.simd[31] = ((10u128) << 64) | 2;
    cpu.simd[30] = ((u64::MAX as u128) << 64) | 3;
    execute(&mut cpu, &mut bus, decode(0x4EFE_87FF).unwrap()).unwrap(); // add v31.2d, v31.2d, v30.2d
    assert_eq!(cpu.simd[31], ((9u128) << 64) | 5);

    cpu.simd[30] = 0x0000_0005_8000_0000_0000_0000_0000_00ff;
    cpu.simd[22] = 0x0000_0008_8000_0000_0000_007b_0000_0f0f;
    execute(&mut cpu, &mut bus, decode(0x4EB6_8FDF).unwrap()).unwrap(); // cmtst v31.4s, v30.4s, v22.4s
    assert_eq!(cpu.simd[31], 0x0000_0000_ffff_ffff_0000_0000_ffff_ffff);

    cpu.simd[29] = 0x1234_5678_0000_0010_8000_0000_ffff_fff0;
    execute(&mut cpu, &mut bus, decode(0x6F3C_07BD).unwrap()).unwrap(); // ushr v29.4s, v29.4s, #4
    assert_eq!(cpu.simd[29], 0x0123_4567_0000_0001_0800_0000_0fff_ffff);

    cpu.simd[31] = 0x99aa_bbcc_ddee_ff00_1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x0EA1_2BEF).unwrap()).unwrap(); // xtn v15.2s, v31.2d
    assert_eq!(cpu.simd[15], 0xddee_ff00_5566_7788);

    cpu.simd[31] = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;
    cpu.simd[28] = 0xaa;
    execute(&mut cpu, &mut bus, decode(0x6E07_079F).unwrap()).unwrap(); // ins v31.b[3], v28.b[0]
    assert_eq!(cpu.simd[31], 0x0011_2233_4455_6677_8899_aabb_aadd_eeff);

    cpu.simd[31] = 0x0f0e_0d0c_0b0a_0908_0706_0504_0302_0100;
    execute(&mut cpu, &mut bus, decode(0x6E20_0BFF).unwrap()).unwrap(); // rev32 v31.16b, v31.16b
    assert_eq!(cpu.simd[31], 0x0c0d_0e0f_0809_0a0b_0405_0607_0001_0203);

    cpu.simd[30] = 0x8000_0001_0001_0001;
    execute(&mut cpu, &mut bus, decode(0x0F2D_57C2).unwrap()).unwrap(); // shl v2.2s, v30.2s, #13
    assert_eq!(cpu.simd[2], 0x0000_2000_2000_2000);

    cpu.simd[4] = 0x0000_0040_0000_007f_0000_0002_0000_0001;
    cpu.simd[6] = 0x0000_0000_5555_5555_aaaa_aaaa_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x6F39_5486).unwrap()).unwrap(); // sli v6.4s, v4.4s, #25
    assert_eq!(cpu.simd[6], 0x8000_0000_ff55_5555_04aa_aaaa_03ff_ffff);

    cpu.simd[27] = 0x8877_6655_4433_2211_0123_4567_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x4E08_077D).unwrap()).unwrap(); // dup v29.2d, v27.d[0]
    assert_eq!(cpu.simd[29], 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

    cpu.simd[29] = 0x8000_0000_0000_0000_0000_0000_0000_0010;
    cpu.simd[25] = 0xffff_ffff_ffff_fffc_0000_0000_0000_0004;
    execute(&mut cpu, &mut bus, decode(0x6EF9_47BD).unwrap()).unwrap(); // ushl v29.2d, v29.2d, v25.2d
    assert_eq!(cpu.simd[29], 0x0800_0000_0000_0000_0000_0000_0000_0100);

    cpu.simd[30] = 0x0000_0004_0000_0003_0000_0002_0000_0001;
    cpu.simd[28] = 0x0000_0008_0000_0007_0000_0006_0000_0005;
    execute(&mut cpu, &mut bus, decode(0x4E9C_1BDE).unwrap()).unwrap(); // uzp1 v30.4s, v30.4s, v28.4s
    assert_eq!(cpu.simd[30], 0x0000_0007_0000_0005_0000_0003_0000_0001);
}

fn vector_bytes(offset: u64) -> u128 {
    let mut value = 0u128;
    for lane in 0..16u64 {
        value |= ((lane + offset) as u128) << (lane * 8);
    }
    value
}

fn f64_lane(cpu: &Armv8Cpu, reg: usize) -> f64 {
    f64::from_bits(cpu.simd[reg] as u64)
}

fn f32_lane(cpu: &Armv8Cpu, reg: usize) -> f32 {
    f32::from_bits(cpu.simd[reg] as u32)
}

#[test]
fn simd_word_immediates_and_cmeq_zero() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x0F00_043F).unwrap()).unwrap(); // movi v31.2s, #1
    assert_eq!(cpu.simd[31], 0x0000_0001_0000_0001);

    execute(&mut cpu, &mut bus, decode(0x2F00_051E).unwrap()).unwrap(); // mvni v30.2s, #8
    assert_eq!(cpu.simd[30], 0xffff_fff7_ffff_fff7);

    cpu.simd[0] = 0x0001_0000_00ff_0000;
    execute(&mut cpu, &mut bus, decode(0x0E20_9800).unwrap()).unwrap(); // cmeq v0.8b, v0.8b, #0
    assert_eq!(cpu.simd[0], 0xff00_ffff_ff00_ffff);
}

#[test]
fn scalar_fp_busybox_arithmetic_and_conversion_ops() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x1E6E_1000).unwrap()).unwrap(); // fmov d0, #1
    assert_eq!(f64_lane(&cpu, 0), 1.0);

    execute(&mut cpu, &mut bus, decode(0x1E62_900F).unwrap()).unwrap(); // fmov d15, #5
    assert_eq!(f64_lane(&cpu, 15), 5.0);

    cpu.regs.set_w(0, 8);
    execute(&mut cpu, &mut bus, decode(0x1E42_F800).unwrap()).unwrap(); // scvtf d0, w0, #2
    assert_eq!(f64_lane(&cpu, 0), 2.0);

    cpu.regs.set_w(20, (-2i32) as u32);
    execute(&mut cpu, &mut bus, decode(0x1E22_0280).unwrap()).unwrap(); // scvtf s0, w20
    assert_eq!(f32_lane(&cpu, 0), -2.0);

    cpu.simd[0] = 1.5f64.to_bits() as u128;
    cpu.simd[31] = 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E7F_0800).unwrap()).unwrap(); // fmul d0, d0, d31
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[25] = 0.25f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_2BFF).unwrap()).unwrap(); // fadd d31, d31, d25
    assert_eq!(f64_lane(&cpu, 31), 2.25);

    cpu.simd[28] = 4.0f64.to_bits() as u128;
    cpu.simd[27] = 1.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E7B_3B9A).unwrap()).unwrap(); // fsub d26, d28, d27
    assert_eq!(f64_lane(&cpu, 26), 2.5);

    cpu.simd[31] = 6.0f64.to_bits() as u128;
    cpu.simd[0] = 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E60_1BE0).unwrap()).unwrap(); // fdiv d0, d31, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    execute(&mut cpu, &mut bus, decode(0x1E61_401F).unwrap()).unwrap(); // fneg d31, d0
    assert_eq!(f64_lane(&cpu, 31), -3.0);

    execute(&mut cpu, &mut bus, decode(0x1E60_C000).unwrap()).unwrap(); // fabs d0, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[0] = 9.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E61_C000).unwrap()).unwrap(); // fsqrt d0, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[0] = 2.25f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E62_401F).unwrap()).unwrap(); // fcvt s31, d0
    assert_eq!(f32_lane(&cpu, 31), 2.25);

    cpu.simd[0] = 1.5f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E22_C000).unwrap()).unwrap(); // fcvt d0, s0
    assert_eq!(f64_lane(&cpu, 0), 1.5);

    cpu.simd[0] = (-2.25f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E65_4000).unwrap()).unwrap(); // frintm d0, d0
    assert_eq!(f64_lane(&cpu, 0), -3.0);

    cpu.regs.set_w(1, 7);
    execute(&mut cpu, &mut bus, decode(0x1E63_003F).unwrap()).unwrap(); // ucvtf d31, w1
    assert_eq!(f64_lane(&cpu, 31), 7.0);

    cpu.regs.set_x(0, 1u64 << 40);
    execute(&mut cpu, &mut bus, decode(0x9E63_001F).unwrap()).unwrap(); // ucvtf d31, x0
    assert_eq!(f64_lane(&cpu, 31), (1u64 << 40) as f64);

    cpu.regs.set_w(0, 6);
    execute(&mut cpu, &mut bus, decode(0x1E03_FC00).unwrap()).unwrap(); // ucvtf s0, w0, #1
    assert_eq!(f32_lane(&cpu, 0), 3.0);

    cpu.simd[31] = (-3.9f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E78_03E0).unwrap()).unwrap(); // fcvtzs w0, d31
    assert_eq!(cpu.regs.w(0), (-3i32) as u32);

    cpu.simd[31] = 3.9f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_03E0).unwrap()).unwrap(); // fcvtzu w0, d31
    assert_eq!(cpu.regs.w(0), 3);

    cpu.simd[0] = 5.9f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E79_0000).unwrap()).unwrap(); // fcvtzu x0, d0
    assert_eq!(cpu.regs.x(0), 5);

    cpu.simd[28] = 2.0f64.to_bits() as u128;
    cpu.simd[27] = 3.0f64.to_bits() as u128;
    cpu.simd[30] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F5B_7B9E).unwrap()).unwrap(); // fmadd d30, d28, d27, d30
    assert_eq!(f64_lane(&cpu, 30), 10.0);

    cpu.simd[29] = 2.0f64.to_bits() as u128;
    cpu.simd[31] = 3.0f64.to_bits() as u128;
    cpu.simd[30] = 10.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F5F_FBBE).unwrap()).unwrap(); // fmsub d30, d29, d31, d30
    assert_eq!(f64_lane(&cpu, 30), 4.0);
}

#[test]
fn scalar_fp_compare_select_and_widening_simd_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[31] = 0.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E60_23E8).unwrap()).unwrap(); // fcmp d31, #0.0
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.simd[29] = 3.0f64.to_bits() as u128;
    cpu.simd[25] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_23B0).unwrap()).unwrap(); // fcmpe d29, d25
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.c());

    cpu.simd[31] = 11.0f64.to_bits() as u128;
    cpu.simd[30] = 22.0f64.to_bits() as u128;
    cpu.pstate.set_nzcv(false, true, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E7E_0FFF).unwrap()).unwrap(); // fcsel d31, d31, d30, eq
    assert_eq!(f64_lane(&cpu, 31), 11.0);

    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E7E_0FFF).unwrap()).unwrap();
    assert_eq!(f64_lane(&cpu, 31), 22.0);

    cpu.simd[0] = u128::MAX;
    execute(&mut cpu, &mut bus, decode(0x6F00_E400).unwrap()).unwrap(); // movi v0.2d, #0
    assert_eq!(cpu.simd[0], 0);

    cpu.simd[31] = 0xffff_ffff_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x2F20_A7FF).unwrap()).unwrap(); // ushll v31.2d, v31.2s, #0
    assert_eq!(cpu.simd[31], 0x0000_0000_ffff_ffff_0000_0000_0000_0002);
}

#[test]
fn simd_strlen_prefix_matches_debian_libc_fast_path() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x4000;

    for (offset, bytes, expected) in [
        (0u64, b"/\0".as_slice(), 1u64),
        (3u64, b"sys\0".as_slice(), 3u64),
    ] {
        for i in 0..32u64 {
            bus.write(base + i, 1, 0xaa);
        }
        for (i, byte) in bytes.iter().enumerate() {
            bus.write(base + offset + i as u64, 1, *byte as u64);
        }

        cpu.regs.set_x(0, base + offset);
        execute(&mut cpu, &mut bus, decode(0x927C_EC01).unwrap()).unwrap(); // and x1, x0, #~0xf
        execute(&mut cpu, &mut bus, decode(0x4C40_7020).unwrap()).unwrap(); // ld1 {v0.16b}, [x1]
        execute(&mut cpu, &mut bus, decode(0x4E20_9801).unwrap()).unwrap(); // cmeq v1.16b, v0.16b, #0
        execute(&mut cpu, &mut bus, decode(0xD37E_F404).unwrap()).unwrap(); // lsl x4, x0, #2
        execute(&mut cpu, &mut bus, decode(0x0F0C_8422).unwrap()).unwrap(); // shrn v2.8b, v1.8h, #4
        execute(&mut cpu, &mut bus, decode(0x9E66_0042).unwrap()).unwrap(); // fmov x2, d2
        execute(&mut cpu, &mut bus, decode(0x9AC4_2442).unwrap()).unwrap(); // lsr x2, x2, x4
        assert_ne!(cpu.regs.x(2), 0);
        execute(&mut cpu, &mut bus, decode(0xDAC0_0042).unwrap()).unwrap(); // rbit x2, x2
        execute(&mut cpu, &mut bus, decode(0xDAC0_1040).unwrap()).unwrap(); // clz x0, x2
        execute(&mut cpu, &mut bus, decode(0xD342_FC00).unwrap()).unwrap(); // lsr x0, x0, #2

        assert_eq!(cpu.regs.x(0), expected);
    }
}

#[test]
fn simd_strlen_page_boundary_matches_debian_libc_path() {
    let (mut cpu, mut bus) = setup();
    let page = RAM_BASE + 0x5000;
    let ptr = page + 0xff0;

    for (bytes, expected) in [
        (b"/\0".as_slice(), 1u64),
        (b"sys\0".as_slice(), 3u64),
        (b"devices\0".as_slice(), 7u64),
    ] {
        for i in 0xfe0..0x1000u64 {
            bus.write(page + i, 1, 0xaa);
        }
        for (i, byte) in bytes.iter().enumerate() {
            bus.write(ptr + i as u64, 1, *byte as u64);
        }

        cpu.regs.set_x(0, ptr);
        execute(&mut cpu, &mut bus, decode(0x927B_E801).unwrap()).unwrap(); // and x1, x0, #~0x1f
        execute(&mut cpu, &mut bus, decode(0x5281_8062).unwrap()).unwrap(); // mov w2, #0xc03
        execute(&mut cpu, &mut bus, decode(0x72B8_0602).unwrap()).unwrap(); // movk w2, #0xc030, lsl #16
        execute(&mut cpu, &mut bus, decode(0x4C40_A021).unwrap()).unwrap(); // ld1 {v1.16b, v2.16b}, [x1]
        execute(&mut cpu, &mut bus, decode(0x4E04_0C40).unwrap()).unwrap(); // dup v0.4s, w2
        execute(&mut cpu, &mut bus, decode(0x4E20_9821).unwrap()).unwrap(); // cmeq v1.16b, v1.16b, #0
        execute(&mut cpu, &mut bus, decode(0x4E20_9842).unwrap()).unwrap(); // cmeq v2.16b, v2.16b, #0
        execute(&mut cpu, &mut bus, decode(0x4E20_1C21).unwrap()).unwrap(); // and v1.16b, v1.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x4E20_1C42).unwrap()).unwrap(); // and v2.16b, v2.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x4E22_BC20).unwrap()).unwrap(); // addp v0.16b, v1.16b, v2.16b
        execute(&mut cpu, &mut bus, decode(0x4E20_BC00).unwrap()).unwrap(); // addp v0.16b, v0.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x9E66_0003).unwrap()).unwrap(); // fmov x3, d0
        execute(&mut cpu, &mut bus, decode(0xD37F_F804).unwrap()).unwrap(); // lsl x4, x0, #1
        execute(&mut cpu, &mut bus, decode(0x9AC4_2463).unwrap()).unwrap(); // lsr x3, x3, x4
        assert_ne!(cpu.regs.x(3), 0);
        execute(&mut cpu, &mut bus, decode(0xDAC0_0063).unwrap()).unwrap(); // rbit x3, x3
        execute(&mut cpu, &mut bus, decode(0xDAC0_1060).unwrap()).unwrap(); // clz x0, x3
        execute(&mut cpu, &mut bus, decode(0xD341_FC00).unwrap()).unwrap(); // lsr x0, x0, #1

        assert_eq!(cpu.regs.x(0), expected);
    }
}

#[test]
fn branch_forward_4_bytes() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0x1400_0002).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn bl_sets_lr_and_jumps() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0x9400_0002).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(30), 0x4000_0004);
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn ret_returns_to_lr() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(30, 0x4000_0100);
    execute(&mut cpu, &mut bus, decode(0xD65F03C0).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0100);
}

#[test]
fn bfm_branch_immediate_insert_preserves_opcode_bits() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(0, 0x1400_0000);
    cpu.regs.set_w(1, 0x3c);

    execute(&mut cpu, &mut bus, decode(0x3302_6C20).unwrap()).unwrap();

    assert_eq!(cpu.regs.w(0), 0x1400_000f);
}

#[test]
fn cbz_branches_when_zero() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 0);
    execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn cbz_falls_through_when_nonzero() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 1);
    execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

#[test]
fn ldp_loads_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0000);
    bus.mem.write(0x4000_0000, 8, 0xDEAD_BEEF);
    bus.mem.write(0x4000_0008, 8, 0xCAFE_BABE);
    execute(&mut cpu, &mut bus, decode(0xA940_0C22).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xDEAD_BEEF);
    assert_eq!(cpu.regs.x(3), 0xCAFE_BABE);
}

#[test]
fn scalar_load_store_translate_each_page_when_crossing_boundary() {
    let (mut cpu, mut bus) = setup();
    let va = 0x1ffc;
    let first_pa = RAM_BASE + 0x0100_0000;
    let second_pa = RAM_BASE + 0x0200_0000;
    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);

    bus.mem.write(first_pa + 0xffc, 4, 0x5566_7788);
    bus.mem.write(second_pa, 4, 0x1122_3344);
    cpu.regs.set_x(1, va);
    execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap(); // ldr x2, [x1]
    assert_eq!(cpu.regs.x(2), 0x1122_3344_5566_7788);

    bus.mem.write(first_pa + PAGE_SIZE, 4, 0xDEAD_BEEF);
    cpu.regs.set_x(0, 0xAABB_CCDD_EEFF_0011);
    execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap(); // str x0, [x1]

    assert_eq!(bus.mem.read(first_pa + 0xffc, 4), Some(0xEEFF_0011));
    assert_eq!(bus.mem.read(second_pa, 4), Some(0xAABB_CCDD));
    assert_eq!(bus.mem.read(first_pa + PAGE_SIZE, 4), Some(0xDEAD_BEEF));
}

#[test]
fn ldpsw_loads_and_sign_extends_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(19, 0x4000_0000);
    bus.mem.write(0x4000_0064, 4, 0xffff_fffc);
    bus.mem.write(0x4000_0068, 4, 0x7fff_fffe);

    execute(&mut cpu, &mut bus, decode(0x694C_9262).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(2), 0xffff_ffff_ffff_fffc);
    assert_eq!(cpu.regs.x(4), 0x7fff_fffe);
}

#[test]
fn mov_reg_copies_value() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x1234_5678);
    execute(&mut cpu, &mut bus, decode(0xAA01_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x1234_5678);
}

#[test]
fn add_imm_adds_constant() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 10);
    execute(&mut cpu, &mut bus, decode(0x9100_0420).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 11);
}

#[test]
fn movk_merges_value() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0xDEAD_BEEF_0000_0000);
    execute(&mut cpu, &mut bus, decode(0xF282_4680).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0xDEAD_BEEF_0000_1234);
}

#[test]
fn adrp_sets_page_relative() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0400;
    execute(&mut cpu, &mut bus, decode(0x9000_0000).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x4000_0000);
}

#[test]
fn tbz_branches_when_bit_clear() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 0b110);
    execute(&mut cpu, &mut bus, decode(0x3600_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

#[test]
fn cmp_sets_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 10);
    cpu.regs.set_x(3, 5);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.n());
}

#[test]
fn cmp_equal_sets_z() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 5);
    cpu.regs.set_x(3, 5);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.n());
}

#[test]
fn cmp_less_than_sets_n() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 3);
    cpu.regs.set_x(3, 10);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(!cpu.pstate.n());
    assert!(!cpu.pstate.z());
}

#[test]
fn cmp_extended_uses_sp_base() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    cpu.regs.set_x(2, 0x4000_0000);

    execute(&mut cpu, &mut bus, decode(0xEB22_63FF).unwrap()).unwrap();

    assert!(cpu.pstate.z());
}

#[test]
fn cmp_immediate_uses_sp_base() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x10;

    execute(&mut cpu, &mut bus, decode(0xF100_43FF).unwrap()).unwrap();

    assert!(cpu.pstate.z());
}

#[test]
fn str_wzr_sp_60() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0xB900_3FFF).unwrap()).unwrap();
    assert_eq!(bus.mem.read(0x4000_003C, 4), Some(0));
}

#[test]
fn dc_zva_zeroes_aligned_block() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(3, RAM_BASE + 13);
    bus.mem.write(RAM_BASE, 8, 0x1111_2222_3333_4444);
    bus.mem.write(RAM_BASE + 8, 8, 0x5555_6666_7777_8888);
    bus.mem.write(RAM_BASE + 16, 8, 0x9999_AAAA_BBBB_CCCC);

    let instr = decode(0xD50B_7423).unwrap();
    assert_eq!(instr.op, Opcode::DcZva);
    execute(&mut cpu, &mut bus, instr).unwrap();

    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 16, 8), Some(0x9999_AAAA_BBBB_CCCC));
}

#[test]
fn ldr_str_roundtrip() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0000);
    cpu.regs.set_x(0, 0xCAFE_0000_DEAD_BEEF);
    execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap();
    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xCAFE_0000_DEAD_BEEF));
    execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xCAFE_0000_DEAD_BEEF);
}

#[test]
fn ccmp_immediate_compares_literal() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(5, 0);
    cpu.pstate.set_nzcv(false, false, false, false); // GE is true
    execute(&mut cpu, &mut bus, decode(0x7A40_A8A0).unwrap()).unwrap();
    assert!(cpu.pstate.z());
}

#[test]
fn ccmn_immediate_adds_literal() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(11, 0xffff_fff8);
    cpu.pstate.set_nzcv(false, true, false, false); // EQ is true
    execute(&mut cpu, &mut bus, decode(0x3A48_0960).unwrap()).unwrap();
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());
}

#[test]
fn ldrsw_sign_extends_to_x_register() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    bus.mem.write(0x4000_0024, 4, 0xffff_fffc);
    execute(&mut cpu, &mut bus, decode(0xB980_27F9).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(25), 0xffff_ffff_ffff_fffc);
}

#[test]
fn daifset_and_daifclr_update_irq_mask() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_42DF).unwrap()).unwrap();
    assert!(cpu.pstate.irq_masked());

    execute(&mut cpu, &mut bus, decode(0xD503_42FF).unwrap()).unwrap();
    assert!(!cpu.pstate.irq_masked());
}

#[test]
fn mrs_daif_reads_current_interrupt_mask() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(true);

    execute(&mut cpu, &mut bus, decode(0xD53B_4220).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0) & (1 << PSTATE_I_BIT), 1 << PSTATE_I_BIT);
}

#[test]
fn extr_executes_32_bit_rotate_alias() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(1, 0x1234_5678);

    execute(&mut cpu, &mut bus, decode(0x1381_0820).unwrap()).unwrap();

    assert_eq!(cpu.regs.w(0), 0x1234_5678u32.rotate_right(2));
}

#[test]
fn extr_executes_32_bit_register_pair_extract() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(3, 0x1122_3344);
    cpu.regs.set_w(4, 0x5566_7788);

    execute(&mut cpu, &mut bus, decode(0x1384_1C62).unwrap()).unwrap();

    let expected = (0x5566_7788u32 >> 7) | (0x1122_3344u32 << 25);
    assert_eq!(cpu.regs.w(2), expected);
}

#[test]
fn msr_daif_restores_all_daif_mask_bits() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);
    cpu.regs.set_x(0, PSTATE_DAIF_MASK);

    execute(
        &mut cpu,
        &mut bus,
        Instr {
            op: Opcode::Msr,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: SYSREG_DAIF as u64,
            sf: true,
            cond: 0,
            size: 0,
        },
    )
    .unwrap();

    assert_eq!(cpu.pstate.daif(), PSTATE_DAIF_MASK);
    assert!(cpu.pstate.irq_masked());
}

#[test]
fn svc_from_el0_sets_syndrome_and_banks_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.sp = 0x7000_0000;
    cpu.sys.sp_el1 = 0x8000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD400_2461).unwrap()).unwrap(); // svc #0x123

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_LOWER_EL_AARCH64);
    assert_eq!(cpu.sys.elr_el1, 0x4000_0004);
    assert_eq!(cpu.sys.spsr_el1 & PSTATE_EL_MASK, 0);
    assert_eq!(cpu.sys.esr_el1 >> 26, ESR_EC_SVC64);
    assert_eq!(cpu.sys.esr_el1 & 0xffff, 0x123);
    assert_eq!(cpu.sys.sp_el0, 0x7000_0000);
    assert_eq!(cpu.regs.sp, 0x8000_0000);
    assert_eq!(cpu.pstate.el(), 1);
    assert_eq!(cpu.pstate.daif(), PSTATE_DAIF_MASK);
}

#[test]
fn eret_to_el0_restores_user_stack_and_saves_kernel_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0xffff_8000_8000_0400;
    cpu.regs.sp = 0xffff_0000_0000_8000;
    cpu.sys.sp_el0 = 0x0000_ffff_ff00_7000;
    cpu.sys.elr_el1 = 0x0000_aaaa_bbbb_c000;
    cpu.sys.spsr_el1 = cpu.pstate.with_el(0).with_irq_masked(false).to_u64();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(true);

    execute(&mut cpu, &mut bus, decode(0xD69F_03E0).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, 0x0000_aaaa_bbbb_c000);
    assert_eq!(cpu.pstate.el(), 0);
    assert_eq!(cpu.regs.sp, 0x0000_ffff_ff00_7000);
    assert_eq!(cpu.sys.sp_el1, 0xffff_0000_0000_8000);
}

#[test]
fn timer_irq_uses_current_el_spx_vector() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.sys.cntp_tval_el0 = 0;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL);
    assert!(cpu.sys.irq_pending);
    assert_eq!(cpu.sys.last_irq_id, PHYSICAL_TIMER_IRQ_ID);
}

#[test]
fn timer_irq_from_el0_uses_lower_vector_and_banks_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.sp = 0x7000_0000;
    cpu.sys.sp_el1 = 0x8000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_LOWER_EL_AARCH64);
    assert_eq!(cpu.sys.spsr_el1 & PSTATE_EL_MASK, 0);
    assert_eq!(cpu.sys.sp_el0, 0x7000_0000);
    assert_eq!(cpu.regs.sp, 0x8000_0000);
    assert!(cpu.sys.irq_pending);
    assert_eq!(cpu.sys.last_irq_id, PHYSICAL_TIMER_IRQ_ID);
}

#[test]
fn virtual_timer_irq_uses_virtual_ppi() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL);
    assert!(cpu.sys.irq_pending);
    assert_eq!(cpu.sys.last_irq_id, VIRTUAL_TIMER_IRQ_ID);
}

#[test]
fn virtual_timer_sysregs_track_tval_ctl_and_cntkctl() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;
    cpu.sys.write_sys_reg(SYSREG_CNTV_TVAL_EL0, 25);
    cpu.sys.write_sys_reg(SYSREG_CNTV_CTL_EL0, TIMER_CTL_ENABLE);
    cpu.sys.write_sys_reg(SYSREG_CNTKCTL_EL1, 0x1234);

    assert_eq!(cpu.sys.cntv_cval_el0, 125);
    assert_eq!(
        cpu.sys.read_sys_reg(SYSREG_CNTV_CTL_EL0, 1),
        TIMER_CTL_ENABLE
    );
    assert_eq!(cpu.sys.read_sys_reg(SYSREG_CNTKCTL_EL1, 1), 0x1234);
}

#[test]
fn timer_tval_reads_count_down_and_accept_signed_deadlines() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;

    cpu.sys.write_sys_reg(SYSREG_CNTP_TVAL_EL0, 25);
    cpu.sys.cycle_count = 110;
    assert_eq!(cpu.sys.read_sys_reg(SYSREG_CNTP_TVAL_EL0, 1), 15);

    cpu.sys.write_sys_reg(SYSREG_CNTV_TVAL_EL0, u32::MAX as u64);
    assert_eq!(cpu.sys.cntv_cval_el0, 109);
    assert_eq!(
        cpu.sys.read_sys_reg(SYSREG_CNTV_TVAL_EL0, 1),
        u32::MAX as u64
    );
}

#[test]
fn masked_timers_do_not_wake_wfi_deadline() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_cval_el0 = 125;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE | TIMER_CTL_IMASK;

    assert_eq!(cpu.sys.next_timer_deadline(), None);
}

#[test]
fn disabled_timer_does_not_deliver_irq() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, 0x4000_0004);
    assert!(!cpu.sys.irq_pending);
}

#[test]
fn casa_updates_memory_on_match_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x4000_0000);
    cpu.regs.set_w(1, 0x1111_2222);
    cpu.regs.set_w(2, 0x3333_4444);
    bus.mem.write(0x4000_0000, 4, 0x1111_2222);

    execute(&mut cpu, &mut bus, decode(0x88E1_7C02).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 4), Some(0x3333_4444));
    assert_eq!(cpu.regs.x(1), 0x1111_2222);
}

#[test]
fn caspal_updates_pair_on_match_and_returns_old_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(4, 0x4000_0000);
    cpu.regs.set_x(0, 0x1111_2222_3333_4444);
    cpu.regs.set_x(1, 0x5555_6666_7777_8888);
    cpu.regs.set_x(2, 0xAAAA_BBBB_CCCC_DDDD);
    cpu.regs.set_x(3, 0xEEEE_FFFF_0000_1111);
    bus.mem.write(0x4000_0000, 8, 0x1111_2222_3333_4444);
    bus.mem.write(0x4000_0008, 8, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0x4860_FC82).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xAAAA_BBBB_CCCC_DDDD));
    assert_eq!(bus.mem.read(0x4000_0008, 8), Some(0xEEEE_FFFF_0000_1111));
    assert_eq!(cpu.regs.x(0), 0x1111_2222_3333_4444);
    assert_eq!(cpu.regs.x(1), 0x5555_6666_7777_8888);
}

#[test]
fn ldaddal_adds_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x4000_0000);
    cpu.regs.set_w(1, 5);
    bus.mem.write(0x4000_0000, 4, 7);

    execute(&mut cpu, &mut bus, decode(0xB8E1_0001).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 4), Some(12));
    assert_eq!(cpu.regs.x(1), 7);
}

#[test]
fn ldseta_sets_bits_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(19, 0x4000_0000);
    cpu.regs.set_x(0, 0b1010);
    bus.mem.write(0x4000_0000, 8, 0b0101);

    execute(&mut cpu, &mut bus, decode(0xF8A0_3260).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0b1111));
    assert_eq!(cpu.regs.x(0), 0b0101);
}

#[test]
fn ldxp_stlxp_pair_roundtrip() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 0x4000_0000);
    cpu.regs.set_x(0, 0xAAAA);
    cpu.regs.set_x(1, 0xBBBB);
    cpu.reserve_exclusive(0x4000_0000, 16);

    execute(&mut cpu, &mut bus, decode(0xC823_8440).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xAAAA));
    assert_eq!(bus.mem.read(0x4000_0008, 8), Some(0xBBBB));
    assert_eq!(cpu.regs.x(3), 0);

    cpu.regs.set_x(0, 0);
    cpu.regs.set_x(1, 0);
    execute(&mut cpu, &mut bus, decode(0xC87F_8440).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0xAAAA);
    assert_eq!(cpu.regs.x(1), 0xBBBB);
}
