use super::*;

#[test]
fn sve_shift_immediates_update_lanes_and_merge_predicated() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    set_z_elem(&mut cpu, 7, 0, 1);
    set_z_elem(&mut cpu, 7, 1, 2);
    execute(&mut cpu, &mut bus, decode(0x04FF_9CE6).unwrap()).unwrap(); // lsl z6.d, z7.d, #63
    assert_eq!(z_elem(&cpu, 6, 0), 0x8000_0000_0000_0000);
    assert_eq!(z_elem(&cpu, 6, 1), 0);

    set_z_elem(&mut cpu, 15, 0, 0x8000_0000_0000_0000);
    set_z_elem(&mut cpu, 15, 1, 0x7FFF_FFFF_FFFF_FFFF);
    execute(&mut cpu, &mut bus, decode(0x04A1_95EE).unwrap()).unwrap(); // lsr z14.d, z15.d, #63
    assert_eq!(z_elem(&cpu, 14, 0), 1);
    assert_eq!(z_elem(&cpu, 14, 1), 0);

    set_z_byte(&mut cpu, 17, 0, 0x80);
    set_z_byte(&mut cpu, 17, 1, 0x7E);
    execute(&mut cpu, &mut bus, decode(0x042F_9230).unwrap()).unwrap(); // asr z16.b, z17.b, #1
    assert_eq!(z_byte(&cpu, 16, 0), 0xC0);
    assert_eq!(z_byte(&cpu, 16, 1), 0x3F);

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 27, 0, 1);
    set_z_elem(&mut cpu, 27, 1, 2);
    execute(&mut cpu, &mut bus, decode(0x04C3_8FFB).unwrap()).unwrap(); // lsl z27.d, p3/m, z27.d, #63
    assert_eq!(z_elem(&cpu, 27, 0), 0x8000_0000_0000_0000);
    assert_eq!(z_elem(&cpu, 27, 1), 2);
}

fn set_z_byte(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u8) {
    cpu.sve_z[reg][lane] = value;
    sync_simd_alias(cpu, reg);
}

fn z_byte(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u8 {
    cpu.sve_z[reg][lane]
}
