use super::*;

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
fn ldr_literal_loads_gpr_and_simd_registers() {
    let (mut cpu, mut bus) = setup();
    let pc = RAM_BASE + 0x3000;

    cpu.regs.pc = pc;
    bus.mem.write(pc, 4, 0xCAFE_BABE);
    execute(&mut cpu, &mut bus, decode(0x1800_0003).unwrap()).unwrap(); // ldr w3, #0
    assert_eq!(cpu.regs.w(3), 0xCAFE_BABE);
    assert_eq!(cpu.regs.x(3) >> 32, 0);

    cpu.regs.pc = pc;
    bus.mem.write(pc, 8, 0x0123_4567_89AB_CDEF);
    execute(&mut cpu, &mut bus, decode(0x5800_0004).unwrap()).unwrap(); // ldr x4, #0
    assert_eq!(cpu.regs.x(4), 0x0123_4567_89AB_CDEF);

    cpu.regs.pc = pc;
    bus.mem.write(pc, 4, 0xFFFF_FFFC);
    execute(&mut cpu, &mut bus, decode(0x9800_0005).unwrap()).unwrap(); // ldrsw x5, #0
    assert_eq!(cpu.regs.x(5), 0xFFFF_FFFF_FFFF_FFFC);

    cpu.regs.pc = pc;
    bus.mem.write(pc, 4, 0x3F80_0000);
    execute(&mut cpu, &mut bus, decode(0x1C00_0006).unwrap()).unwrap(); // ldr s6, #0
    assert_eq!(cpu.simd[6], 0x3F80_0000);

    cpu.regs.pc = pc;
    bus.mem.write(pc, 8, 0x4008_0000_0000_0000);
    execute(&mut cpu, &mut bus, decode(0x5C00_0007).unwrap()).unwrap(); // ldr d7, #0
    assert_eq!(cpu.simd[7], 0x4008_0000_0000_0000);

    cpu.regs.pc = pc;
    bus.mem.write(pc, 8, 0x0706_0504_0302_0100);
    bus.mem.write(pc + 8, 8, 0x0F0E_0D0C_0B0A_0908);
    execute(&mut cpu, &mut bus, decode(0x9C00_0008).unwrap()).unwrap(); // ldr q8, #0
    assert_eq!(cpu.simd[8], 0x0F0E_0D0C_0B0A_0908_0706_0504_0302_0100);
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
fn cache_maintenance_and_prfm_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(2, RAM_BASE + 0x1000);

    execute(&mut cpu, &mut bus, decode(0xD50B_7B22).unwrap()).unwrap();
    execute(&mut cpu, &mut bus, decode(0xF8A0_6AB0).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, RAM_BASE + 8);
    assert_eq!(cpu.regs.x(2), RAM_BASE + 0x1000);
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
