use super::*;

#[test]
fn sve_logical_immediates_update_64_bit_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    set_z_elem(&mut cpu, 0, 0, 0xffff);
    set_z_elem(&mut cpu, 0, 1, 0x1234);
    execute(&mut cpu, &mut bus, decode(0x0582_00E0).unwrap()).unwrap(); // and z0.d, z0.d, #0xff
    assert_eq!(z_elem(&cpu, 0, 0), 0xff);
    assert_eq!(z_elem(&cpu, 0, 1), 0x34);

    set_z_elem(&mut cpu, 1, 0, 0x5500_5500_5500_5500);
    execute(&mut cpu, &mut bus, decode(0x0500_04E1).unwrap()).unwrap(); // orr z1.h, z1.h, #0xff
    assert_eq!(z_elem(&cpu, 1, 0), 0x55ff_55ff_55ff_55ff);

    set_z_elem(&mut cpu, 2, 0, 0xffff_0000_ffff_0000);
    execute(&mut cpu, &mut bus, decode(0x0540_44E2).unwrap()).unwrap(); // eor z2.h, z2.h, #0xff00
    assert_eq!(z_elem(&cpu, 2, 0), 0x00ff_ff00_00ff_ff00);

    execute(&mut cpu, &mut bus, decode(0x05C2_00E3).unwrap()).unwrap(); // dupm z3.d, #0xff
    assert!((0..4).all(|lane| z_elem(&cpu, 3, lane) == 0xff));
    assert_eq!(cpu.simd[3], (0xffu128 << 64) | 0xff);
}
