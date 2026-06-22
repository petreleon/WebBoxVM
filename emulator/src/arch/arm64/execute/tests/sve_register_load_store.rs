use super::*;

const STR_Z10_SP: u32 = 0xE580_43EA;
const SVE_VA_PAGE: u64 = 0x1000;
const SVE_FIRST_PA: u64 = RAM_BASE + 0x0700_0000;
const SVE_SECOND_PA: u64 = RAM_BASE + 0x0800_0000;

#[test]
fn sve_register_load_store_forms_transfer_z_and_predicate_bytes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    cpu.regs.sp = RAM_BASE + 0x2000;

    for lane in 0..4 {
        set_z_elem(&mut cpu, 10, lane, 0x1111_0000_0000_0000 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0xE580_43EA).unwrap()).unwrap(); // str z10, [sp]
    for byte in 0..32 {
        assert_eq!(
            bus.read(cpu.regs.sp + byte, 1),
            Some(cpu.sve_z[10][byte as usize] as u64)
        );
    }

    cpu.sve_z[8] = [0; 256];
    cpu.simd[8] = 0;
    execute(&mut cpu, &mut bus, decode(0x8580_43E8).unwrap()).unwrap(); // ldr z8, [sp]
    assert!((0..4).all(|lane| z_elem(&cpu, 8, lane) == z_elem(&cpu, 10, lane)));
    assert_eq!(cpu.simd[8], cpu.simd[10]);

    for byte in 0..32 {
        bus.write(cpu.regs.sp + 32 + byte, 1, 0x80 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x8580_47E8).unwrap()).unwrap(); // ldr z8, [sp, #1, mul vl]
    assert_eq!(cpu.sve_z[8][0], 0x80);
    assert_eq!(cpu.sve_z[8][31], 0x9f);

    cpu.sve_pred[4] = [0x0000_0000_0000_80A5, 0, 0, 0];
    execute(&mut cpu, &mut bus, decode(0xE580_03E4).unwrap()).unwrap(); // str p4, [sp]
    assert_eq!(bus.read(cpu.regs.sp, 1), Some(0xA5));
    assert_eq!(bus.read(cpu.regs.sp + 1, 1), Some(0x80));
    assert_eq!(bus.read(cpu.regs.sp + 2, 1), Some(0));
    assert_eq!(bus.read(cpu.regs.sp + 3, 1), Some(0));

    cpu.sve_pred[11] = [u64::MAX; 4];
    execute(&mut cpu, &mut bus, decode(0x8580_03EB).unwrap()).unwrap(); // ldr p11, [sp]
    assert_eq!(cpu.sve_pred[11][0], 0x80A5);
    assert_eq!(cpu.sve_pred[11][1], 0);

    cpu.sve_pred[4] = [0x55AA, 0, 0, 0];
    execute(&mut cpu, &mut bus, decode(0xE5BF_1FE4).unwrap()).unwrap(); // str p4, [sp, #-1, mul vl]
    assert_eq!(bus.read(cpu.regs.sp - 4, 1), Some(0xAA));
    assert_eq!(bus.read(cpu.regs.sp - 3, 1), Some(0x55));

    execute(&mut cpu, &mut bus, decode(0xE5BF_5FEA).unwrap()).unwrap(); // str z10, [sp, #-1, mul vl]
    for byte in 0..32 {
        assert_eq!(
            bus.read(cpu.regs.sp - 32 + byte, 1),
            Some(cpu.sve_z[10][byte as usize] as u64)
        );
    }
}

#[test]
fn sve_same_page_vector_store_clears_overlapping_exclusive() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    cpu.regs.sp = RAM_BASE + 0x7000;
    for lane in 0..4 {
        set_z_elem(&mut cpu, 10, lane, 0x2222_0000_0000_0000 + lane as u64);
    }
    cpu.reserve_exclusive(cpu.regs.sp + 20, 8);

    execute(&mut cpu, &mut bus, decode(STR_Z10_SP).unwrap()).unwrap();

    assert_eq!(bus.read(cpu.regs.sp, 1), Some(cpu.sve_z[10][0] as u64));
    assert_eq!(
        bus.read(cpu.regs.sp + 31, 1),
        Some(cpu.sve_z[10][31] as u64)
    );
    assert!(cpu.exclusive.is_none());
}

#[test]
fn sve_cross_page_vector_store_preserves_partial_fault_order() {
    let (mut cpu, mut bus) = setup();
    let va = SVE_VA_PAGE + PAGE_SIZE - 2;
    cpu.sve_vl_bytes = 16;
    cpu.regs.sp = va;
    map_two_user_pages(&mut cpu, &mut bus, SVE_VA_PAGE, SVE_FIRST_PA, SVE_SECOND_PA);
    unmap_second_sve_page(&mut bus);
    set_z_elem(&mut cpu, 10, 0, 0xa7a6_a5a4_a3a2_a1a0);
    set_z_elem(&mut cpu, 10, 1, 0xafae_adac_abaa_a9a8);
    for offset in 0..4u64 {
        bus.mem.write(SVE_SECOND_PA + offset, 1, 0xcc);
    }

    let err = execute(&mut cpu, &mut bus, decode(STR_Z10_SP).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    assert_eq!(bus.mem.read(SVE_FIRST_PA + PAGE_SIZE - 2, 1), Some(0xa0));
    assert_eq!(bus.mem.read(SVE_FIRST_PA + PAGE_SIZE - 1, 1), Some(0xa1));
    assert_eq!(bus.mem.read(SVE_SECOND_PA, 1), Some(0xcc));
}

fn unmap_second_sve_page(bus: &mut SystemBus) {
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let l3_idx = (SVE_VA_PAGE >> PT_L3_SHIFT) & 0x1ff;
    bus.mem.write(l3 + (l3_idx + 1) * 8, 8, 0);
}
