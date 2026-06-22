use super::simd_helpers::*;
use super::*;

const LD4_V28_16B_X1: u32 = 0x4C40_003C;
const VA_PAGE: u64 = 0x1000;
const FIRST_PA: u64 = RAM_BASE + 0x0100_0000;
const SECOND_PA: u64 = RAM_BASE + 0x0200_0000;
const UNREADABLE_PA: u64 = 0x9000_0000;

#[test]
fn simd_structured_load_reads_across_page_boundary() {
    let (mut cpu, mut bus) = setup();
    let va = map_cross_page_ld4(&mut cpu, &mut bus, SECOND_PA);
    seed_cross_page_ld4(&mut bus, SECOND_PA);
    cpu.regs.set_x(1, va);

    execute(&mut cpu, &mut bus, decode(LD4_V28_16B_X1).unwrap()).unwrap();

    for reg in 0..4 {
        assert_eq!(
            cpu.simd[28 + reg],
            ld_structure_vector_bytes(0, 4, reg as u64, 1, 16)
        );
    }
}

#[test]
fn simd_structured_load_fault_before_register_update_on_missing_page() {
    let (mut cpu, mut bus) = setup();
    let va = map_cross_page_ld4(&mut cpu, &mut bus, SECOND_PA);
    unmap_second_page(&mut bus);
    seed_first_page_tail(&mut bus);
    cpu.regs.set_x(1, va);
    let before = preserve_target_vectors(&mut cpu);

    let err = execute(&mut cpu, &mut bus, decode(LD4_V28_16B_X1).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    assert_target_vectors(&cpu, before);
}

#[test]
fn simd_structured_load_fault_before_register_update_on_unreadable_page() {
    let (mut cpu, mut bus) = setup();
    let va = map_cross_page_ld4(&mut cpu, &mut bus, UNREADABLE_PA);
    seed_first_page_tail(&mut bus);
    cpu.regs.set_x(1, va);
    let before = preserve_target_vectors(&mut cpu);

    let err = execute(&mut cpu, &mut bus, decode(LD4_V28_16B_X1).unwrap()).unwrap_err();

    assert_eq!(err, "LD4 bus fault");
    assert_target_vectors(&cpu, before);
}

fn map_cross_page_ld4(cpu: &mut Armv8Cpu, bus: &mut SystemBus, second_pa: u64) -> u64 {
    map_two_user_pages(cpu, bus, VA_PAGE, FIRST_PA, second_pa);
    VA_PAGE + PAGE_SIZE - 32
}

fn seed_cross_page_ld4(bus: &mut SystemBus, second_pa: u64) {
    seed_first_page_tail(bus);
    for offset in 32..64u64 {
        bus.mem.write(second_pa + offset - 32, 1, offset);
    }
}

fn seed_first_page_tail(bus: &mut SystemBus) {
    for offset in 0..32u64 {
        bus.mem.write(FIRST_PA + PAGE_SIZE - 32 + offset, 1, offset);
    }
}

fn unmap_second_page(bus: &mut SystemBus) {
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let l3_idx = (VA_PAGE >> PT_L3_SHIFT) & 0x1ff;
    bus.mem.write(l3 + (l3_idx + 1) * 8, 8, 0);
}

fn preserve_target_vectors(cpu: &mut Armv8Cpu) -> [u128; 4] {
    let before = [
        0x1111_2222_3333_4444,
        0x5555_6666_7777_8888,
        0x9999_aaaa_bbbb_cccc,
        0xdddd_eeee_ffff_0000,
    ];
    cpu.simd[28..32].copy_from_slice(&before);
    before
}

fn assert_target_vectors(cpu: &Armv8Cpu, before: [u128; 4]) {
    assert_eq!(cpu.simd[28..32], before);
}
