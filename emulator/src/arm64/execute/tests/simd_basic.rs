use super::*;

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
fn simd_umov_and_smov_extend_elements_to_gpr() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[30] = 0x7777_6666_5555_4444_3333_2222_1111_abcd;
    cpu.regs.set_x(0, u64::MAX);

    execute(&mut cpu, &mut bus, decode(0x0E02_3FC0).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0xabcd);

    cpu.simd[31] = 0x7777_6666_5555_4444_3333_8000_1111_7fff;
    cpu.regs.set_x(2, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0x0E0A_2FE2).unwrap()).unwrap(); // smov w2, v31.h[2]
    assert_eq!(cpu.regs.x(2), 0x0000_0000_ffff_8000);

    cpu.simd[31] = 0x7777_7777_7777_7777_0000_0002_8000_0001;
    cpu.regs.set_x(0, 0);
    execute(&mut cpu, &mut bus, decode(0x4E04_2FE0).unwrap()).unwrap(); // smov x0, v31.s[0]
    assert_eq!(cpu.regs.x(0), 0xffff_ffff_8000_0001);
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

    cpu.simd[31] = 0x0000_0005_ffff_ffff;
    cpu.simd[30] = 0xffff_fffe_0000_0007;
    execute(&mut cpu, &mut bus, decode(0x2EBE_6FFF).unwrap()).unwrap(); // umin v31.2s, v31.2s, v30.2s
    assert_eq!(cpu.simd[31], 0x0000_0005_0000_0007);

    cpu.simd[29] = 0x0000_0004_0000_0003_ffff_ffff_0000_0001;
    cpu.simd[31] = 0x0000_0000_ffff_ffff_0000_0005_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x6EBF_67BD).unwrap()).unwrap(); // umax v29.4s, v29.4s, v31.4s
    assert_eq!(cpu.simd[29], 0x0000_0004_ffff_ffff_ffff_ffff_0000_0002);

    cpu.simd[30] = 0x0000_0003_8000_0001;
    cpu.simd[31] = 0xffff_fff0_7fff_ffff;
    execute(&mut cpu, &mut bus, decode(0x0EBF_67DF).unwrap()).unwrap(); // smax v31.2s, v30.2s, v31.2s
    assert_eq!(cpu.simd[31], 0x0000_0003_7fff_ffff);

    execute(&mut cpu, &mut bus, decode(0x4E22_BC45).unwrap()).unwrap(); // addp v5.16b, v2.16b, v2.16b
    assert_eq!(cpu.simd[5], 0x0307_0b0f_1317_1b1f_0307_0b0f_1317_1b1f);

    cpu.simd[28] = 0x0000_0000_0000_0003_ffff_ffff_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x5EF1_BB9F).unwrap()).unwrap(); // addp d31, v28.2d
    assert_eq!(cpu.simd[31], 2);

    cpu.simd[15] = u64::MAX as u128;
    cpu.simd[31] = 2;
    execute(&mut cpu, &mut bus, decode(0x5EFF_85EF).unwrap()).unwrap(); // add d15, d15, d31
    assert_eq!(cpu.simd[15], 1);
}
