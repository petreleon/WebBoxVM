use super::*;
use crate::constants::*;

#[test]
fn rejects_undecodable_instruction_at_start_pc() {
    let mut cpu = Armv8Cpu::default();
    let mut bus = SystemBus::new();
    cpu.regs.pc = RAM_BASE;
    bus.mem.write(RAM_BASE, 4, 0xffff_ffff);
    bus.mem.write(RAM_BASE + 4, 4, 0xd503_201f);

    assert_eq!(
        block_from_pc(&cpu, &bus).err(),
        Some("block starts with undecodable instruction")
    );
}

#[test]
fn rejects_unreadable_instruction_at_start_pc() {
    let mut cpu = Armv8Cpu::default();
    let bus = SystemBus::new();
    cpu.regs.pc = 0x9000_0000;

    assert_eq!(
        block_from_pc(&cpu, &bus).err(),
        Some("block instruction read fault")
    );
}

#[test]
fn fetch_cache_retranslates_after_page_boundary() {
    let mut cpu = Armv8Cpu::default();
    let mut bus = SystemBus::new();
    map_user_page_one(&mut cpu, &mut bus, RAM_BASE + 0x3000);
    cpu.regs.pc = 0x1ffc;
    bus.mem.write(RAM_BASE + 0x3ffc, 4, 0xd503_201f);
    bus.mem.write(RAM_BASE + 0x4000, 4, 0xd503_201f);

    let block = block_from_pc(&cpu, &bus).expect("first mapped instruction should compile");

    assert_eq!(block.instruction_pas, vec![RAM_BASE + 0x3ffc]);
    assert_eq!(block.instructions.len(), 1);
}

#[test]
fn hvc_terminates_block_before_following_instruction() {
    let mut cpu = Armv8Cpu::default();
    let mut bus = SystemBus::new();
    cpu.regs.pc = RAM_BASE;
    bus.mem.write(RAM_BASE, 4, 0xd503_201f);
    bus.mem.write(RAM_BASE + 4, 4, 0xd402_4682);
    bus.mem.write(RAM_BASE + 8, 4, 0xd503_201f);

    let block = block_from_pc(&cpu, &bus).expect("HVC block should decode");

    assert_eq!(block.instructions.len(), 2);
    assert_eq!(block.instructions[1].0.op, Opcode::Hvc);
    assert_eq!(block.instructions[1].0.imm, 0x1234);
}

fn map_user_page_one(cpu: &mut Armv8Cpu, bus: &mut SystemBus, pa: u64) {
    let l1 = RAM_BASE;
    let l2 = RAM_BASE + PAGE_SIZE;
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    bus.mem.write(l1, 8, (l2 & DESC_ADDR_MASK) | DESC_TABLE);
    bus.mem.write(l2, 8, (l3 & DESC_ADDR_MASK) | DESC_TABLE);
    bus.mem
        .write(l3 + 8, 8, (pa & DESC_ADDR_MASK) | DESC_VALID | DESC_AF_BIT);
    cpu.sys.ttbr0_el1 = l1;
    cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
}
