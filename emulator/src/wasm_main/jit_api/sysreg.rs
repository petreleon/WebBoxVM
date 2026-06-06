use crate::arm64::machine::Machine;
use crate::constants::SYSREG_SP_EL0;
use crate::wasm_main::Emulator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Read a side-effect-free system register for a generated JIT block.
    pub fn jit_read_sysreg(&mut self, core_id: Option<usize>, sysreg_id: u32) -> u64 {
        if self.jit_helper_failed {
            return 0;
        }
        let core_id = core_id.unwrap_or(0);
        let sysreg_id = sysreg_id as u16;
        let result = if let Some(ref mut boot) = self.boot {
            jit_read_sysreg_from_machine(&mut boot.machine, core_id, sysreg_id)
        } else {
            jit_read_sysreg_from_machine(&mut self.machine, core_id, sysreg_id)
        };

        match result {
            Ok(value) => value,
            Err(err) => {
                self.jit_last_error = err;
                self.jit_helper_failed = true;
                0
            }
        }
    }
}

pub(super) fn jit_read_sysreg_from_machine(
    machine: &mut Machine,
    core_id: usize,
    sysreg_id: u16,
) -> Result<u64, String> {
    let cpu = machine
        .cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    if sysreg_id != SYSREG_SP_EL0 {
        return Err(format!("JIT sysreg helper rejected sysreg 0x{sysreg_id:04x}"));
    }
    Ok(cpu.sys.read_sys_reg(sysreg_id, cpu.pstate.el()))
}
