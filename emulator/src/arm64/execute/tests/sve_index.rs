use super::*;

#[test]
fn sve_index_builds_signed_lane_sequence() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    cpu.regs.set_x(9, 10);
    cpu.regs.set_x(10, 3);
    execute(&mut cpu, &mut bus, decode(0x04AA_4D3F).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 31, 0), 10);
    assert_eq!(z_word(&cpu, 31, 1), 13);
    assert_eq!(z_word(&cpu, 31, 7), 31);

    cpu.regs.set_x(20, 0xFFFF_FFFE);
    execute(&mut cpu, &mut bus, decode(0x04A1_4690).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 16, 0), 0xFFFF_FFFE);
    assert_eq!(z_word(&cpu, 16, 1), 0xFFFF_FFFF);
    assert_eq!(z_word(&cpu, 16, 2), 0);
    assert_eq!(z_word(&cpu, 16, 3), 1);
}

#[test]
fn sve_index_handles_immediate_forms() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;

    execute(&mut cpu, &mut bus, decode(0x04A3_43C4).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 4, 0), 0xFFFF_FFFE);
    assert_eq!(z_word(&cpu, 4, 1), 1);
    assert_eq!(z_word(&cpu, 4, 2), 4);

    cpu.regs.set_x(7, 4);
    execute(&mut cpu, &mut bus, decode(0x04A7_4B82).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 2, 0), 0xFFFF_FFFC);
    assert_eq!(z_word(&cpu, 2, 1), 0);
    assert_eq!(z_word(&cpu, 2, 2), 4);
}
