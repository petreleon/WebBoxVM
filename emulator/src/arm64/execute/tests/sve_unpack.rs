use super::*;

#[test]
fn sve_vector_unpack_widens_low_and_high_halves() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;

    for lane in 0..16 {
        set_z_byte(&mut cpu, 2, lane, lane as u8);
    }
    execute(&mut cpu, &mut bus, decode(0x0572_3841).unwrap()).unwrap(); // uunpklo z1.h, z2.b
    assert_eq!(z_half(&cpu, 1, 0), 0);
    assert_eq!(z_half(&cpu, 1, 7), 7);

    execute(&mut cpu, &mut bus, decode(0x0573_3841).unwrap()).unwrap(); // uunpkhi z1.h, z2.b
    assert_eq!(z_half(&cpu, 1, 0), 8);
    assert_eq!(z_half(&cpu, 1, 7), 15);

    set_z_half(&mut cpu, 3, 0, 0x8001);
    execute(&mut cpu, &mut bus, decode(0x05B0_3861).unwrap()).unwrap(); // sunpklo z1.s, z3.h
    assert_eq!(z_word(&cpu, 1, 0), 0xFFFF_8001);
}

#[test]
fn sve_predicate_unpack_widens_b_to_h_predicate_bits() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    cpu.sve_pred[7] = [0b1000_0001_0000_0101, 0, 0, 0];

    execute(&mut cpu, &mut bus, decode(0x0530_40E1).unwrap()).unwrap(); // punpklo p1.h, p7.b
    assert!(pred_bit(&cpu, 1, 0));
    assert!(!pred_bit(&cpu, 1, 1));
    assert!(pred_bit(&cpu, 1, 4));
    assert!(!pred_bit(&cpu, 1, 6));

    execute(&mut cpu, &mut bus, decode(0x0531_40E2).unwrap()).unwrap(); // punpkhi p2.h, p7.b
    assert!(pred_bit(&cpu, 2, 0));
    assert!(!pred_bit(&cpu, 2, 2));
}

fn set_z_byte(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u8) {
    cpu.sve_z[reg][lane] = value;
    sync_simd_alias(cpu, reg);
}

fn set_z_half(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u16) {
    let offset = lane * 2;
    cpu.sve_z[reg][offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn z_half(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u16 {
    let offset = lane * 2;
    let mut bytes = [0; 2];
    bytes.copy_from_slice(&cpu.sve_z[reg][offset..offset + 2]);
    u16::from_le_bytes(bytes)
}
