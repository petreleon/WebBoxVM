//! Debug dump helpers for breakpoint diagnostics.

use crate::arch::arm64::{Armv8Cpu, SystemRegisters, Tlb, decode, translate};
use crate::constants::LINK_REGISTER_INDEX;
use crate::platform::virt::SystemBus;

pub(crate) fn dump_breakpoint_context(cpu: &Armv8Cpu, bus: &SystemBus, pc: u64) {
    dump_instructions("PC", pc, cpu, bus);
    dump_instructions("LR", cpu.regs.x(LINK_REGISTER_INDEX), cpu, bus);
    dump_string_pointers(cpu, bus);
    dump_stack(cpu, bus);
}

fn dump_instructions(label: &str, addr: u64, cpu: &Armv8Cpu, bus: &SystemBus) {
    let mut scratch_tlb = Tlb::new();
    eprintln!("Instructions around {} ({:#018x}):", label, addr);
    for offset in (-32i64..=32).step_by(4) {
        let target = offset_addr(addr, offset);
        if let Ok(pa) = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, target)
            && let Some(val) = bus.mem.read(pa, 4)
        {
            let decoded = decode(val as u32);
            eprintln!(
                "  {:#018x}: {:08x} {:?}",
                target,
                val,
                decoded.map(|d| d.op)
            );
        }
    }
}

fn dump_string_pointers(cpu: &Armv8Cpu, bus: &SystemBus) {
    for (i, &reg_val) in [
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
    ]
    .iter()
    .enumerate()
    {
        if reg_val == 0 {
            continue;
        }
        let mut scratch_tlb = Tlb::new();
        if let Some(s) = try_read_string_at(bus, &mut scratch_tlb, &cpu.sys, reg_val)
            && !s.is_empty()
            && s.len() > 2
        {
            eprintln!("  maybe @X{}: \"{}\"", i, s);
        }
    }
}

fn try_read_string_at(
    bus: &SystemBus,
    tlb: &mut Tlb,
    sys: &SystemRegisters,
    addr: u64,
) -> Option<String> {
    let mut s = String::new();
    for off in 0..128u64 {
        match translate(sys, tlb, &bus.mem, addr + off) {
            Ok(pa) => {
                if let Some(val) = bus.mem.read(pa, 1) {
                    let byte = val as u8;
                    if byte == 0 {
                        break;
                    }
                    if byte.is_ascii_graphic() || byte == b' ' {
                        s.push(byte as char);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    if s.is_empty() { None } else { Some(s) }
}

fn dump_stack(cpu: &Armv8Cpu, bus: &SystemBus) {
    let sp = cpu.regs.sp;
    eprintln!("Stack around SP={:#018x}:", sp);
    let mut scratch_tlb = Tlb::new();
    for offset in (-64i64..=64).step_by(8) {
        let addr = offset_addr(sp, offset);
        if let Ok(pa) = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, addr)
            && let Some(val) = bus.mem.read(pa, 8)
        {
            eprintln!("  {:#018x}: {:016x}", addr, val);
        }
    }
}

fn offset_addr(addr: u64, offset: i64) -> u64 {
    (addr as i64).wrapping_add(offset) as u64
}
