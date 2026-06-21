use super::Emulator;
use crate::arch::arm64::{decode, translate};
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Return the current instruction context for browser-side diagnosis.
    pub fn current_instruction(&mut self, core_id: Option<usize>) -> String {
        let core_id = core_id.unwrap_or(0);
        if let Some(ref mut boot) = self.boot {
            return current_instruction_for(&mut boot.machine, core_id);
        }
        current_instruction_for(&mut self.machine, core_id)
    }

    pub fn debug_translate_va(&mut self, va: u64, core_id: Option<usize>) -> String {
        translate_va_for(active_machine_mut(self), core_id.unwrap_or(0), va)
    }

    pub fn debug_read_va_u64(&mut self, va: u64, core_id: Option<usize>) -> String {
        read_va_u64_for(active_machine_mut(self), core_id.unwrap_or(0), va)
    }

    pub fn debug_read_pa_u64(&mut self, pa: u64) -> String {
        read_pa_u64_for(active_machine_mut(self), pa)
    }
}

fn active_machine_mut(emulator: &mut Emulator) -> &mut Machine {
    if let Some(ref mut boot) = emulator.boot {
        &mut boot.machine
    } else {
        &mut emulator.machine
    }
}

fn current_instruction_for(machine: &mut Machine, core_id: usize) -> String {
    if core_id >= machine.cpus.len() {
        return format!(r#"{{"error":"core {core_id} does not exist"}}"#);
    }

    let cpu = &mut machine.cpus[core_id];
    let pc = cpu.regs.pc;
    let el = cpu.pstate.el();
    let pa = match translate(&cpu.sys, &mut cpu.tlb, &machine.bus.mem, pc) {
        Ok(pa) => pa,
        Err(_) => return format!(r#"{{"pc":"0x{pc:016x}","el":{el},"error":"translate"}}"#),
    };
    let raw = match machine.bus.mem.read_u32(pa) {
        Some(raw) => raw,
        None => {
            return format!(r#"{{"pc":"0x{pc:016x}","pa":"0x{pa:016x}","error":"fetch"}}"#);
        }
    };

    match decode(raw) {
        Some(instr) => format!(
            r#"{{"pc":"0x{pc:016x}","pa":"0x{pa:016x}","el":{el},"raw":"0x{raw:08x}","opcode":"{}","opcodeId":{}}}"#,
            instr.op.name(),
            instr.op.id()
        ),
        None => format!(
            r#"{{"pc":"0x{pc:016x}","pa":"0x{pa:016x}","el":{el},"raw":"0x{raw:08x}","opcode":"Undecoded"}}"#
        ),
    }
}

fn translate_va_for(machine: &mut Machine, core_id: usize, va: u64) -> String {
    if core_id >= machine.cpus.len() {
        return format!(r#"{{"error":"core {core_id} does not exist"}}"#);
    }

    let cpu = &mut machine.cpus[core_id];
    match translate(&cpu.sys, &mut cpu.tlb, &machine.bus.mem, va) {
        Ok(pa) => format!(r#"{{"va":"0x{va:016x}","pa":"0x{pa:016x}"}}"#),
        Err(_) => format!(r#"{{"va":"0x{va:016x}","error":"translate"}}"#),
    }
}

fn read_va_u64_for(machine: &mut Machine, core_id: usize, va: u64) -> String {
    if core_id >= machine.cpus.len() {
        return format!(r#"{{"error":"core {core_id} does not exist"}}"#);
    }

    let cpu = &mut machine.cpus[core_id];
    let pa = match translate(&cpu.sys, &mut cpu.tlb, &machine.bus.mem, va) {
        Ok(pa) => pa,
        Err(_) => return format!(r#"{{"va":"0x{va:016x}","error":"translate"}}"#),
    };
    match machine.bus.mem.read_u64(pa) {
        Some(value) => {
            format!(r#"{{"va":"0x{va:016x}","pa":"0x{pa:016x}","value":"0x{value:016x}"}}"#)
        }
        None => format!(r#"{{"va":"0x{va:016x}","pa":"0x{pa:016x}","error":"read"}}"#),
    }
}

fn read_pa_u64_for(machine: &mut Machine, pa: u64) -> String {
    match machine.bus.mem.read_u64(pa) {
        Some(value) => format!(r#"{{"pa":"0x{pa:016x}","value":"0x{value:016x}"}}"#),
        None => format!(r#"{{"pa":"0x{pa:016x}","error":"read"}}"#),
    }
}
