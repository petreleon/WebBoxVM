//! WebAssembly entry point: multi-core ARM64 emulator + kernel boot.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

use crate::arm64::Machine;
use crate::boot::BootContext;
use wasm_bindgen::prelude::*;

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
    /// `kernel_image`: raw bytes of an ARM64 Linux Image
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

    /// Load and configure a Linux kernel Image with a caller-provided initrd.
    pub fn boot_kernel_with_initrd(
        &mut self,
        kernel_image: Vec<u8>,
        initrd: Vec<u8>,
        num_cores: usize,
    ) -> String {
        match BootContext::new_with_initrd(&kernel_image, num_cores, &initrd) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(ctx);
                format!(
                    "OK: kernel loaded with custom initrd, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load and configure a Linux kernel Image with a caller-provided BusyBox binary.
    pub fn boot_kernel_with_busybox(
        &mut self,
        kernel_image: Vec<u8>,
        busybox: Vec<u8>,
        num_cores: usize,
    ) -> String {
        match BootContext::new_with_busybox(&kernel_image, num_cores, &busybox) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(ctx);
                format!(
                    "OK: kernel loaded with BusyBox initrd, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load an ARM64 Linux ISO by extracting its kernel/initrd boot pair.
    pub fn boot_iso(&mut self, iso_image: Vec<u8>, num_cores: usize) -> String {
        match BootContext::new_from_iso_owned(iso_image, num_cores) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(ctx);
                format!("OK: ISO kernel/initrd loaded, {} cores ready", cores)
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load an ARM64 Linux ISO and configure a writable sparse install disk.
    pub fn boot_iso_with_disk(
        &mut self,
        iso_image: Vec<u8>,
        num_cores: usize,
        disk_size_bytes: u64,
    ) -> String {
        match BootContext::new_from_iso_owned(iso_image, num_cores) {
            Ok(mut ctx) => {
                ctx.set_install_disk_size(disk_size_bytes);
                let cores = ctx.machine.cpus.len();
                self.boot = Some(ctx);
                format!(
                    "OK: ISO kernel/initrd loaded with writable disk, {} cores ready",
                    cores
                )
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

    /// Get UART output length in bytes.
    pub fn uart_output_len(&self) -> usize {
        if let Some(ref boot) = self.boot {
            boot.uart_output_len()
        } else {
            self.machine.bus.uart.output.len()
        }
    }

    /// Get UART output since a byte offset.
    pub fn uart_output_since(&self, offset: usize) -> String {
        if let Some(ref boot) = self.boot {
            boot.uart_output_since(offset)
        } else {
            let output = &self.machine.bus.uart.output;
            String::from_utf8_lossy(&output[offset.min(output.len())..]).to_string()
        }
    }

    /// Send text to the guest UART receive path.
    pub fn send_uart_input(&mut self, input: &str) {
        if let Some(ref mut boot) = self.boot {
            boot.feed_uart_input(input);
        } else {
            self.machine.bus.uart.feed_input(input);
            self.machine.inject_irq(crate::constants::PL011_UART_IRQ_ID);
        }
    }

    /// Send raw bytes to the guest UART receive path.
    pub fn send_uart_bytes(&mut self, input: Vec<u8>) {
        if let Some(ref mut boot) = self.boot {
            boot.feed_uart_bytes(&input);
        } else if !input.is_empty() {
            self.machine.bus.uart.feed_input_bytes(&input);
            self.machine.inject_irq(crate::constants::PL011_UART_IRQ_ID);
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

    /// Number of allocated guest memory pages.
    pub fn allocated_pages(&self) -> usize {
        if let Some(ref boot) = self.boot {
            boot.allocated_pages()
        } else {
            self.machine.bus.mem.allocated_pages()
        }
    }

    /// Bytes allocated by the sparse writable install disk.
    pub fn install_disk_allocated_bytes(&self) -> u64 {
        if let Some(ref boot) = self.boot {
            boot.install_disk_allocated_bytes()
        } else {
            self.machine.bus.virtio_disk.allocated_storage_bytes()
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
