//! WebAssembly entry point: multi-core ARM64 emulator + kernel boot.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

use wasm_bindgen::prelude::*;
use crate::arm64::Machine;
use crate::boot::BootContext;

/// Multi-core ARM64 Emulator exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    machine: Machine,
    boot: Option<BootContext>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(cores: Option<usize>) -> Emulator {
        Emulator {
            machine: Machine::new(cores.unwrap_or(1)),
            boot: None,
        }
    }

    /// Load and configure a Linux kernel Image for boot.
    /// `kernel_image`: raw bytes of Image.gz or vmlinuz
    /// `num_cores`: number of ARM64 cores to emulate
    pub fn boot_kernel(&mut self, kernel_image: Vec<u8>, num_cores: usize) -> String {
        match BootContext::new(&kernel_image, num_cores) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(ctx);
                format!("OK: kernel loaded, {} cores ready", cores)
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Run the EFI stub phase (bootloader).
    pub fn run_efi(&mut self, max_steps: usize) -> String {
        if let Some(ref mut boot) = self.boot {
            let steps = boot.run_efi_phase(max_steps);
            format!("EFI: {} steps, PC={:#018x}", steps, boot.pc())
        } else {
            "ERR: no kernel loaded".to_string()
        }
    }

    /// Run the kernel phase using the multi-core machine.
    pub fn run_kernel(&mut self, max_steps: usize) -> String {
        if let Some(ref mut boot) = self.boot {
            let steps = boot.run_kernel_phase(max_steps);
            format!("KERNEL: {} steps, PC={:#018x}", steps, boot.pc())
        } else {
            "ERR: no kernel loaded".to_string()
        }
    }

    /// Get UART output.
    pub fn uart_output(&self) -> String {
        if let Some(ref boot) = self.boot {
            boot.uart_output()
        } else {
            self.machine.bus.uart.output_string()
        }
    }

    /// Get register Xn of a core.
    pub fn reg(&self, n: u8, core_id: Option<usize>) -> u64 {
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
        if let Some(ref boot) = self.boot {
            boot.total_steps()
        } else {
            self.machine.total_steps
        }
    }

    /// Get PC of core 0.
    pub fn pc(&self) -> u64 {
        if let Some(ref boot) = self.boot {
            boot.pc()
        } else if !self.machine.cpus.is_empty() {
            self.machine.cpus[0].regs.pc
        } else {
            0
        }
    }
}
