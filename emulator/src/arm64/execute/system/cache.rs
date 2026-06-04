use super::helpers::fault_to_error;
use crate::arm64::helpers::read_reg;
use crate::arm64::mmu::translate_write;
use crate::arm64::{Armv8Cpu, Instr};
use crate::bus::SystemBus;
use crate::constants::DCZID_EL0_VAL;

pub(in crate::arm64::execute) fn exec_dc_zva(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    visit_dc_block(cpu, bus, instr, true)
}

pub(in crate::arm64::execute) fn exec_dc_gva(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    visit_dc_block(cpu, bus, instr, false)
}

fn visit_dc_block(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
    zero_data: bool,
) -> Result<(), &'static str> {
    if DCZID_EL0_VAL & 0x10 != 0 {
        return Ok(());
    }

    let block_size = 4u64 << (DCZID_EL0_VAL & 0xF);
    let base = read_reg(cpu, instr.rd, true) & !(block_size - 1);
    let mut offset = 0;
    while offset < block_size {
        let size = (block_size - offset).min(8) as u8;
        let va = base + offset;
        let pa = translate_write(&cpu.sys, &mut bus.mem, va, cpu.pstate.el()).map_err(|fault| {
            cpu.sys.far_el1 = va;
            fault_to_error(fault)
        })?;
        if zero_data {
            bus.write(pa, size, 0);
        }
        offset += size as u64;
    }
    Ok(())
}
