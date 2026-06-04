use super::*;

#[test]
fn simd_ld1r_replicates_loaded_doubleword() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x3000;
    cpu.regs.set_x(0, base);
    bus.write(base, 8, 0x1122_3344_5566_7788);

    execute(&mut cpu, &mut bus, decode(0x4D40_CC1F).unwrap()).unwrap(); // ld1r {v31.2d}, [x0]

    assert_eq!(cpu.simd[31], 0x1122_3344_5566_7788_1122_3344_5566_7788);

    let post_imm_base = base + 0x20;
    cpu.regs.set_x(10, post_imm_base);
    bus.write(post_imm_base, 8, 0x0102_0304_0506_0708);

    execute(&mut cpu, &mut bus, decode(0x4DDF_CD5A).unwrap()).unwrap(); // ld1r {v26.2d}, [x10], #8

    assert_eq!(cpu.simd[26], 0x0102_0304_0506_0708_0102_0304_0506_0708);
    assert_eq!(cpu.regs.x(10), post_imm_base + 8);

    let post_reg_base = base + 0x40;
    cpu.regs.set_x(0, post_reg_base);
    cpu.regs.set_x(1, 0x18);
    bus.write(post_reg_base, 8, 0x8877_6655_4433_2211);

    execute(&mut cpu, &mut bus, decode(0x4DC1_CC00).unwrap()).unwrap(); // ld1r {v0.2d}, [x0], x1

    assert_eq!(cpu.simd[0], 0x8877_6655_4433_2211_8877_6655_4433_2211);
    assert_eq!(cpu.regs.x(0), post_reg_base + 0x18);
}
