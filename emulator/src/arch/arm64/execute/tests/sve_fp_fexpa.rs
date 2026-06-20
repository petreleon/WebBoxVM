use super::*;

#[test]
fn sve_fexpa_writes_exact_lookup_results_for_all_element_sizes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    set_z_half(&mut cpu, 1, 0, 0x0000);
    set_z_half(&mut cpu, 1, 1, 0x03ff);
    execute(&mut cpu, &mut bus, decode(0x0460_B820).unwrap()).unwrap(); // fexpa z0.h, z1.h
    assert_eq!(z_half(&cpu, 0, 0), 0x0000);
    assert_eq!(z_half(&cpu, 0, 1), 0x7fd4);

    set_z_word(&mut cpu, 2, 0, 0x0000_0000);
    set_z_word(&mut cpu, 2, 7, 0x0000_3fff);
    execute(&mut cpu, &mut bus, decode(0x04A0_B840).unwrap()).unwrap(); // fexpa z0.s, z2.s
    assert_eq!(z_word(&cpu, 0, 0), 0x0000_0000);
    assert_eq!(z_word(&cpu, 0, 7), 0x7ffd_3e0c);

    set_z_elem(&mut cpu, 3, 0, 0x0000_0000_0000_0000);
    set_z_elem(&mut cpu, 3, 3, 0x0000_0000_001f_ffff);
    execute(&mut cpu, &mut bus, decode(0x04E0_B860).unwrap()).unwrap(); // fexpa z0.d, z3.d
    assert_eq!(z_elem(&cpu, 0, 0), 0x0000_0000_0000_0000);
    assert_eq!(z_elem(&cpu, 0, 3), 0x7fff_a7c1_819e_90d8);
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

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
