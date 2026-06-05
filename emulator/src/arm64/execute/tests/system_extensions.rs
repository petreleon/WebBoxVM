use super::*;

#[test]
fn disabled_system_extensions_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(16, 1);
    cpu.regs.set_x(4, 0x4444);
    cpu.regs.set_x(5, 0x5555);

    execute(&mut cpu, &mut bus, decode(0xD503_251F).unwrap()).unwrap(); // chkfeat x16
    execute(&mut cpu, &mut bus, decode(0xD50B_7744).unwrap()).unwrap(); // gcsss1 x4
    execute(&mut cpu, &mut bus, decode(0xD52B_7725).unwrap()).unwrap(); // gcspopm x5
    execute(&mut cpu, &mut bus, decode(0xD503_467F).unwrap()).unwrap(); // smstop

    assert_eq!(cpu.regs.x(16), 1);
    assert_eq!(cpu.regs.x(4), 0x4444);
    assert_eq!(cpu.regs.x(5), 0x5555);
    assert_eq!(cpu.regs.pc, RAM_BASE + 16);
}

#[test]
fn mte_dc_tag_cache_ops_use_tagless_data_behavior() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, RAM_BASE + 13);
    bus.mem.write(RAM_BASE, 8, 0x1111_2222_3333_4444);
    bus.mem.write(RAM_BASE + 8, 8, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0xD50B_7462).unwrap()).unwrap(); // dc gva, x2
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0x1111_2222_3333_4444));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0x5555_6666_7777_8888));

    execute(&mut cpu, &mut bus, decode(0xD50B_7482).unwrap()).unwrap(); // dc gzva, x2
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0));
}

#[test]
fn sysl_writes_zero_result() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(3, 0xFFFF);

    execute(&mut cpu, &mut bus, decode(0xD528_7423).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(3), 0);
}

#[test]
fn pauth_hint_aliases_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(30, 0xCAFE);

    execute(&mut cpu, &mut bus, decode(0xD503_233F).unwrap()).unwrap(); // paciasp
    execute(&mut cpu, &mut bus, decode(0xD503_23BF).unwrap()).unwrap(); // autiasp
    execute(&mut cpu, &mut bus, decode(0xD503_20FF).unwrap()).unwrap(); // xpaclri

    assert_eq!(cpu.regs.x(30), 0xCAFE);
    assert_eq!(cpu.regs.pc, RAM_BASE + 12);
}

#[test]
fn bti_hint_aliases_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(1, 0xB71);

    for raw in [0xD503_241F, 0xD503_245F, 0xD503_249F, 0xD503_24DF] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    assert_eq!(cpu.regs.x(1), 0xB71);
    assert_eq!(cpu.regs.pc, RAM_BASE + 16);
}

#[test]
fn event_hint_aliases_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(2, 0x5E7);

    execute(&mut cpu, &mut bus, decode(0xD503_209F).unwrap()).unwrap(); // sev
    execute(&mut cpu, &mut bus, decode(0xD503_20BF).unwrap()).unwrap(); // sevl

    assert_eq!(cpu.regs.x(2), 0x5E7);
    assert_eq!(cpu.regs.pc, RAM_BASE + 8);
}

#[test]
fn sync_hint_aliases_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(3, 0xC5DB);

    for raw in [
        0xD503_221F,
        0xD503_223F,
        0xD503_225F,
        0xD503_227F,
        0xD503_229F,
        0xD503_22DF,
    ] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    assert_eq!(cpu.regs.x(3), 0xC5DB);
    assert_eq!(cpu.regs.pc, RAM_BASE + 24);
}

#[test]
fn unsupported_128_bit_system_classes_trap() {
    for raw in [0xD548_0000, 0xD570_0000, 0xD550_0000] {
        let (mut cpu, mut bus) = setup();
        cpu.pstate = cpu.pstate.with_el(1);
        cpu.sys.vbar_el1 = RAM_BASE + 0x1000;

        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();

        assert_eq!(cpu.sys.esr_el1, (ESR_EC_UNKNOWN << 26) | ESR_IL);
        assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + 0x200);
    }
}

#[test]
fn flag_system_instructions_update_nzcv() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate.set_nzcv(false, true, true, false);

    execute(&mut cpu, &mut bus, decode(0xD500_401F).unwrap()).unwrap(); // cfinv
    assert!(!cpu.pstate.c());

    cpu.regs.set_x(1, 0b1110);
    execute(&mut cpu, &mut bus, decode(0xBA00_042F).unwrap()).unwrap(); // rmif x1, #0, #15
    assert!(cpu.pstate.n() && cpu.pstate.z() && cpu.pstate.c());
    assert!(!cpu.pstate.v());

    cpu.regs.set_w(1, 0x80);
    execute(&mut cpu, &mut bus, decode(0x3A00_082D).unwrap()).unwrap(); // setf8 w1
    assert!(cpu.pstate.n() && cpu.pstate.v());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.regs.set_w(1, 0);
    execute(&mut cpu, &mut bus, decode(0x3A00_482D).unwrap()).unwrap(); // setf16 w1
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.n() && !cpu.pstate.v());
    assert!(cpu.pstate.c());
}
