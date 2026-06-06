use super::Emulator;
use crate::arm64::{Machine, decode, translate};
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
            return format!(
                r#"{{"pc":"0x{pc:016x}","pa":"0x{pa:016x}","error":"fetch"}}"#
            );
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
