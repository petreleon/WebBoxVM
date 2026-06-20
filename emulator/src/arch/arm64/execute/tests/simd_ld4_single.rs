use super::simd_helpers::*;
use super::*;

#[test]
fn simd_ld4_single_loads_one_lane_into_four_registers() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5000;
    cpu.regs.set_x(0, base);
    for (offset, byte) in [
        0x44, 0x33, 0x22, 0x11, 0x88, 0x77, 0x66, 0x55, 0xcc, 0xbb, 0xaa, 0x99, 0x00, 0xff, 0xee,
        0xdd,
    ]
    .into_iter()
    .enumerate()
    {
        bus.write(base + offset as u64, 1, byte as u64);
    }
    for reg in 4..8 {
        cpu.simd[reg] = u32x4([
            0xaaaa_0000 + reg as u32,
            0xbbbb_0000,
            0xcccc_0000,
            0xdddd_0000,
        ]);
    }

    execute(&mut cpu, &mut bus, decode(0x0DFF_B004).unwrap()).unwrap();

    assert_eq!(
        cpu.simd[4],
        u32x4([0xaaaa_0004, 0x1122_3344, 0xcccc_0000, 0xdddd_0000])
    );
    assert_eq!(
        cpu.simd[5],
        u32x4([0xaaaa_0005, 0x5566_7788, 0xcccc_0000, 0xdddd_0000])
    );
    assert_eq!(
        cpu.simd[6],
        u32x4([0xaaaa_0006, 0x99aa_bbcc, 0xcccc_0000, 0xdddd_0000])
    );
    assert_eq!(
        cpu.simd[7],
        u32x4([0xaaaa_0007, 0xddee_ff00, 0xcccc_0000, 0xdddd_0000])
    );
    assert_eq!(cpu.regs.x(0), base + 16);
}

#[test]
fn simd_ld4_single_register_post_index_uses_rm() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5100;
    cpu.regs.set_x(0, base);
    cpu.regs.set_x(4, 40);
    for byte in 0..16u64 {
        bus.write(base + byte, 1, 0x80 + byte);
    }

    execute(&mut cpu, &mut bus, decode(0x0DE4_A004).unwrap()).unwrap();

    assert_eq!(u32_lane(cpu.simd[4], 0), 0x8382_8180);
    assert_eq!(u32_lane(cpu.simd[7], 0), 0x8f8e_8d8c);
    assert_eq!(cpu.regs.x(0), base + 40);
}
