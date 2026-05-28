use super::*;
use crate::arm64::{Armv8Cpu, decode, execute};
use crate::bus::SystemBus;

#[test]
fn mmu_enabled_virtual_fetch_and_store() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    // Build a 3-level page table for VA 0xFFFF_FF80_0000_0000 -> PA 0x4000_3000
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    // Write code at physical address 0x4000_3000
    let code: [u32; 3] = [
        0xD280_0140, // MOVZ X0, #10
        0xD280_0401, // MOVZ X1, #32
        0x9A01_0002, // ADD X2, X0, X1
    ];
    for (i, &word) in code.iter().enumerate() {
        bus.mem.write(0x4000_3000 + (i as u64 * 4), 4, word as u64);
    }

    // Set up MMU registers
    cpu.sys.ttbr1_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << 16) | 25; // 39-bit VA
    cpu.sys.sctlr_el1 = 1; // Enable MMU

    let va = 0xFFFF_FF80_0000_0000;
    let steps = run(&mut cpu, &mut bus, va, 3).unwrap();
    assert_eq!(steps, 3);
    assert_eq!(cpu.regs.x(2), 42);
}

#[test]
fn mmu_tlb_hit_after_first_miss() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    bus.mem.write(0x4000_3000, 8, 0xD503201F); // NOP

    cpu.sys.ttbr1_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << 16) | 25;
    cpu.sys.sctlr_el1 = 1;

    let va = 0xFFFF_FF80_0000_0000;

    // First fetch should TLB miss and walk
    let steps1 = run(&mut cpu, &mut bus, va, 1).unwrap();
    assert_eq!(steps1, 1);

    // TLB should now have an entry
    assert!(cpu.tlb.lookup(va).is_some());

    // Second run should TLB hit
    let steps2 = run(&mut cpu, &mut bus, va, 1).unwrap();
    assert_eq!(steps2, 1);
}

#[test]
fn mmu_msr_to_ttbr_invalidates_tlb() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    cpu.sys.ttbr1_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << 16) | 25;
    cpu.sys.sctlr_el1 = 1;

    let va = 0xFFFF_FF80_0000_0000;

    // Prime TLB
    let _ = run(&mut cpu, &mut bus, va, 1);
    assert!(cpu.tlb.lookup(va).is_some());

    // MSR to TTBR1_EL1 should invalidate TLB
    cpu.regs.set_x(0, l1_table);
    let instr = decode(0xd5182020).unwrap(); // MSR TTBR1_EL1, X0
    execute(&mut cpu, &mut bus, instr).unwrap();
    assert!(cpu.tlb.lookup(va).is_none());
}

#[test]
fn mmu_tlbi_instruction_invalidates_tlb() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    cpu.sys.ttbr1_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << 16) | 25;
    cpu.sys.sctlr_el1 = 1;

    let va = 0xFFFF_FF80_0000_0000;

    // Prime TLB
    let _ = run(&mut cpu, &mut bus, va, 1);
    assert!(cpu.tlb.lookup(va).is_some());

    // TLBI VMALLE1
    let tlbi = decode(0xd508871f).unwrap();
    execute(&mut cpu, &mut bus, tlbi).unwrap();
    assert!(cpu.tlb.lookup(va).is_none());
}
