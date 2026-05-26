//! Interpreted execution: fetch-decode-execute loop.

use super::{Armv8Cpu, decode, execute};
use crate::bus::SystemBus;

#[cfg(test)]
mod tests;

/// Run the CPU starting at `entry` for exactly `steps` instructions.
pub fn run(cpu: &mut Armv8Cpu, bus: &mut SystemBus, entry: u64, max_steps: usize) -> Result<usize, RunError> {
    cpu.regs.pc = entry;
    for _ in 0..max_steps {
        let raw = fetch32(cpu, bus)?;
        let instr = decode(raw).ok_or(RunError::Decode(raw, cpu.regs.pc))?;
        execute(cpu, bus, instr).map_err(|e| RunError::Exec(e, cpu.regs.pc))?;
    }
    Ok(max_steps)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    Fetch(u64),
    Decode(u32, u64),
    Exec(&'static str, u64),
}

fn fetch32(cpu: &Armv8Cpu, bus: &SystemBus) -> Result<u32, RunError> {
    let word = bus.read(cpu.regs.pc, 4).ok_or(RunError::Fetch(cpu.regs.pc))?;
    Ok(word as u32)
}
