use crate::arm64::{Armv8Cpu, Machine};
use crate::bus::SystemBus;
use crate::constants::*;
use crate::dtb::{build_dtb, load_dtb};
use crate::initrd::{build_cpio, load_initrd};

/// Holds everything needed to boot and run a Linux kernel.
pub struct BootContext {
    pub machine: Machine,
    pub dtb_addr: u64,
}

impl BootContext {
    pub fn new(kernel_image: &[u8], num_cores: usize) -> Result<Self, String> {
        let mut machine = Machine::new(num_cores);

        // Copy kernel image into RAM at KERNEL_LOAD_ADDR
        for (i, &byte) in kernel_image.iter().enumerate() {
            machine.bus.write(KERNEL_LOAD_ADDR + i as u64, 1, byte as u64);
        }

        // Standard ARM64 Linux boot protocol:
        // X0 = physical address of DTB, X1-X3 = 0, MMU off
        let cpu0 = &mut machine.cpus[0];
        cpu0.regs.set_x(0, DTB_BASE);
        cpu0.regs.set_x(1, 0);
        cpu0.regs.set_x(2, 0);
        cpu0.regs.set_x(3, 0);
        cpu0.sys.sctlr_el1 = 0; // MMU disabled — kernel's head.S enables it
        // Jump to ARM64 Image header (code0+cod1 branch to primary_entry)
        cpu0.regs.pc = KERNEL_LOAD_ADDR;

        // Build initrd and DTB
        let initrd = build_minimal_initrd();
        let initrd_end = INITRD_BASE + initrd.len() as u64;
        let dtb = build_dtb(
            RAM_BASE, RAM_SIZE,
            Some(INITRD_BASE), Some(initrd_end),
            Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"),
        );
        load_initrd(&mut machine.bus, INITRD_BASE, &initrd);
        load_dtb(&mut machine.bus, DTB_BASE, &dtb);

        Ok(BootContext {
            machine,
            dtb_addr: DTB_BASE,
        })
    }

    /// No-op: EFI stub is skipped.  We boot via the standard ARM64 protocol.
    pub fn run_efi_phase(&mut self, _max_steps: usize) -> usize {
        0
    }

    /// Run the multi-core kernel phase (round-robin scheduling).
    pub fn run_kernel_phase(&mut self, max_steps: usize) -> usize {
        self.machine.run(max_steps)
    }

    pub fn uart_output(&self) -> String { self.machine.bus.uart.output_string() }
    pub fn total_steps(&self) -> u64 { self.machine.total_steps }
    pub fn pc(&self) -> u64 { self.machine.cpus[0].regs.pc }
}

// ── Boot helpers ──

fn read_kernel_image_size(data: &[u8]) -> u64 {
    if data.len() >= 24 {
        u64::from_le_bytes([
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ])
    } else {
        data.len() as u64
    }
}

fn build_minimal_initrd() -> Vec<u8> {
    let busybox_data = vec![0u8; 100];
    let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nexec /bin/sh\n".to_vec();
    let entries = vec![
        ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
        ("bin/sh".to_string(), busybox_data, 0o100755u32),
        ("init".to_string(), init_script, 0o100755u32),
    ];
    build_cpio(&entries)
}
