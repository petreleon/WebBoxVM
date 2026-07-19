use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Run the EFI stub phase (bootloader).
    pub fn run_efi(&mut self, max_steps: usize) -> String {
        let _access = self.require_parallel_idle();
        if let Some(ref mut boot) = self.boot {
            let steps = boot.run_efi_phase(max_steps);
            format!("EFI: {} steps, PC={:#018x}", steps, boot.pc())
        } else {
            "ERR: no kernel loaded".to_string()
        }
    }

    /// Run the kernel phase using the multi-core machine.
    pub fn run_kernel(&mut self, max_steps: usize) -> String {
        let _access = self.require_parallel_idle();
        if let Some(ref mut boot) = self.boot {
            let steps = boot.run_kernel_phase(max_steps);
            format!("KERNEL: {} steps, PC={:#018x}", steps, boot.pc())
        } else {
            "ERR: no kernel loaded".to_string()
        }
    }

    /// Prepare the serial scheduler and return the next runnable core for JIT.
    ///
    /// Returns -1 when no core is runnable after wake checks and idle-time
    /// fast-forwarding.
    pub fn jit_prepare_next_core(&mut self) -> i32 {
        let _access = self.require_parallel_idle();
        let core = if let Some(ref mut boot) = self.boot {
            boot.machine.prepare_next_core()
        } else {
            self.machine.prepare_next_core()
        };
        core.and_then(|core| i32::try_from(core).ok()).unwrap_or(-1)
    }

    /// Get register Xn of a core.
    pub fn reg(&self, n: u8, core_id: Option<usize>) -> u64 {
        let _access = self.require_parallel_idle();
        let cid = core_id.unwrap_or(0);
        if let Some(ref boot) = self.boot {
            if cid < boot.machine.cpus.len() {
                return boot.machine.cpus[cid].regs.x(n);
            }
        }
        if cid < self.machine.cpus.len() {
            self.machine.cpus[cid].regs.x(n)
        } else {
            0
        }
    }

    /// Total steps across all phases.
    pub fn total_steps(&self) -> u64 {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.total_steps()
        } else {
            self.machine.total_steps
        }
    }

    /// Cooperative-scheduler WFE instructions that entered a sleep state.
    pub fn cooperative_wfe_parks(&self) -> u64 {
        let _access = self.require_parallel_idle();
        let machine = self
            .boot
            .as_ref()
            .map_or(self.machine.as_ref(), |boot| &boot.machine);
        machine.cooperative_wfe_parks
    }

    /// Guest cycles skipped while every cooperative vCPU was asleep.
    pub fn cooperative_idle_fast_forward_cycles(&self) -> u64 {
        let _access = self.require_parallel_idle();
        let machine = self
            .boot
            .as_ref()
            .map_or(self.machine.as_ref(), |boot| &boot.machine);
        machine.cooperative_idle_fast_forward_cycles
    }

    /// Get PC of core 0.
    pub fn pc(&self) -> u64 {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.pc()
        } else if !self.machine.cpus.is_empty() {
            self.machine.cpus[0].regs.pc
        } else {
            0
        }
    }

    /// Get the PC of a specific core, or zero when the core does not exist.
    pub fn pc_for_core(&self, core_id: usize) -> u64 {
        let _access = self.require_parallel_idle();
        let machine = self
            .boot
            .as_ref()
            .map_or(self.machine.as_ref(), |boot| &boot.machine);
        machine.cpus.get(core_id).map_or(0, |cpu| cpu.regs.pc)
    }
}
