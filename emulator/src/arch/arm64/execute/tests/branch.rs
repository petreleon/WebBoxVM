use super::*;

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
fn pac_branch_uses_target_register() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0100);

    execute(&mut cpu, &mut bus, decode(0xD71F_0821).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, 0x4000_0100);
}

#[test]
fn pac_branch_link_sets_lr_and_jumps() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(2, 0x4000_0200);

    execute(&mut cpu, &mut bus, decode(0xD73F_0843).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(30), 0x4000_0004);
    assert_eq!(cpu.regs.pc, 0x4000_0200);
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
