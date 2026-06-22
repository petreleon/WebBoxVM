use crate::host::wasm::Emulator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Copy one emulated core's architectural register state into the JIT state buffer.
    pub fn jit_sync_state_from_core(&mut self, core_id: Option<usize>) -> bool {
        let core_id = core_id.unwrap_or(0);
        self.jit_helper_failed = false;
        self.clear_jit_side_effects();
        let cpu = if let Some(ref boot) = self.boot {
            boot.machine.cpus.get(core_id)
        } else {
            self.machine.cpus.get(core_id)
        };

        let Some(cpu) = cpu else {
            self.jit_last_error = format!("core {core_id} does not exist");
            return false;
        };

        self.jit_state.copy_from_cpu(cpu);
        self.jit_last_error.clear();
        true
    }

    /// Copy the JIT state buffer back into one emulated core.
    ///
    /// This is intentionally explicit so the browser worker can validate a JIT
    /// block before allowing it to mutate the VM.
    pub fn jit_sync_state_to_core(&mut self, core_id: Option<usize>) -> bool {
        let core_id = core_id.unwrap_or(0);
        let cpu = if let Some(ref mut boot) = self.boot {
            boot.machine.cpus.get_mut(core_id)
        } else {
            self.machine.cpus.get_mut(core_id)
        };

        let Some(cpu) = cpu else {
            self.jit_last_error = format!("core {core_id} does not exist");
            return false;
        };

        self.jit_state.copy_to_cpu(cpu);
        self.jit_last_error.clear();
        true
    }
}
