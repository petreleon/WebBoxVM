use super::simd_helpers::vector_bytes;
use super::*;

#[test]
fn sve_byte_load_store_register_offsets_use_scalar_index() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    let base = RAM_BASE + 0x2800;

    cpu.regs.set_x(1, base);
    cpu.regs.set_x(2, 7);
    cpu.sve_pred[1][0] = (1 << 0) | (1 << 2);
    bus.write(base + 7, 1, 0xA0);
    bus.write(base + 9, 1, 0xA2);

    execute(&mut cpu, &mut bus, decode(0xA402_4421).unwrap()).unwrap();
    assert_eq!(cpu.sve_z[1][0], 0xA0);
    assert_eq!(cpu.sve_z[1][1], 0);
    assert_eq!(cpu.sve_z[1][2], 0xA2);

    cpu.simd[1] = vector_bytes(0x10);
    for offset in 0..8 {
        bus.write(base + 0x100 + offset, 1, 0xCC);
    }
    cpu.regs.set_x(1, base + 0x100);
    cpu.regs.set_x(2, 3);
    cpu.sve_pred[1][0] = (1 << 0) | (1 << 3);

    execute(&mut cpu, &mut bus, decode(0xE402_4421).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base + 0x103, 1), Some(0x10));
    assert_eq!(bus.mem.read(base + 0x104, 1), Some(0xCC));
    assert_eq!(bus.mem.read(base + 0x106, 1), Some(0x13));
}
