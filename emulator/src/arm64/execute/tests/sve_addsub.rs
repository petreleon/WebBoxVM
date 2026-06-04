use super::*;

#[test]
fn sve_addsub_immediate_updates_all_lanes_with_element_wrapping() {
    let (mut cpu, mut bus) = setup();
    set_z_word(&mut cpu, 3, 0, 0xFFFF_FFFE);
    set_z_word(&mut cpu, 3, 1, 0x1000_0000);

    execute(&mut cpu, &mut bus, decode(0x25A0_FFE3).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 3, 0), 0x0000_FEFE);
    assert_eq!(z_word(&cpu, 3, 1), 0x1000_FF00);

    set_z_elem(&mut cpu, 7, 0, 0);
    set_z_elem(&mut cpu, 7, 1, 0x1_0000);
    execute(&mut cpu, &mut bus, decode(0x25E1_FFE7).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 7, 0), 0xFFFF_FFFF_FFFF_0100);
    assert_eq!(z_elem(&cpu, 7, 1), 0x100);
}

#[test]
fn sve_addsub_predicated_preserves_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 30, 0, 10);
    set_z_elem(&mut cpu, 30, 1, 20);
    set_z_elem(&mut cpu, 31, 0, 3);
    set_z_elem(&mut cpu, 31, 1, 4);

    execute(&mut cpu, &mut bus, decode(0x04C0_0FFE).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 30, 0), 13);
    assert_eq!(z_elem(&cpu, 30, 1), 20);

    set_z_elem(&mut cpu, 31, 0, 10);
    set_z_elem(&mut cpu, 31, 1, 20);
    set_z_elem(&mut cpu, 29, 0, 3);
    set_z_elem(&mut cpu, 29, 1, 4);
    execute(&mut cpu, &mut bus, decode(0x04C1_0FBF).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 31, 0), 7);
    assert_eq!(z_elem(&cpu, 31, 1), 20);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
