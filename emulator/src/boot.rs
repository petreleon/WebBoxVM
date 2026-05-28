//! Kernel boot API: loads ARM64 Linux kernel Image, sets up EFI/DTB/initrd.
//! Exposed via wasm-bindgen for multi-core boot testing.

use crate::arm64::{Armv8Cpu, Machine, decode, execute, translate, Opcode};
use crate::bus::SystemBus;
use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
use crate::efi::setup_efi_tables;
use crate::dtb::{build_dtb, load_dtb};
use crate::initrd::{build_cpio, load_initrd};

/// Boot context: holds a configured Machine ready to boot Linux.
pub struct BootContext {
    pub machine: Machine,
    pub dtb_addr: u64,
    pub entry_pc: u64,
    efi_stub_done: bool,
    pages_bump: u64,
    history: Vec<(u64, u32, Opcode)>,
}

impl BootContext {
    /// Create a boot context from a kernel image (raw bytes of Image.gz or vmlinuz).
    /// `num_cores`: number of ARM64 CPU cores (default 1).
    pub fn new(kernel_image: &[u8], num_cores: usize) -> Result<Self, String> {
        // Write kernel to memory via SystemBus
        let mut machine = Machine::new(num_cores);
        
        // Load kernel into RAM at KERNEL_LOAD
        for (i, &byte) in kernel_image.iter().enumerate() {
            machine.bus.write(KERNEL_LOAD + i as u64, 1, byte as u64);
        }

        // Parse kernel header for image_size
        let image_size = if kernel_image.len() >= 24 {
            u64::from_le_bytes([
                kernel_image[16], kernel_image[17], kernel_image[18], kernel_image[19],
                kernel_image[20], kernel_image[21], kernel_image[22], kernel_image[23],
            ])
        } else {
            kernel_image.len() as u64
        };

        let dtb_addr = 0x4700_0000u64;
        let (handle, st) = setup_efi_tables(&mut machine.bus, KERNEL_LOAD, image_size, dtb_addr);

        // Configure core 0 as the boot CPU
        let cpu0 = &mut machine.cpus[0];
        cpu0.regs.set_x(0, handle);
        cpu0.regs.set_x(1, st);
        cpu0.regs.sp = 0x43F0_0000;

        // Set up return trampoline (RET at 0x43EFE000)
        machine.bus.write(0x43EFE000, 4, 0xD65F03C0);
        cpu0.regs.set_x(30, 0x43EFE000);

        // Set up initial page tables for MMU
        setup_boot_page_tables(cpu0, &mut machine.bus);

        // PE entry point for EFI stub
        let entry = KERNEL_LOAD + 0x01da7ee0;
        cpu0.regs.pc = entry;

        // Build minimal initrd (busybox)
        let busybox_data = vec![0u8; 100]; // dummy busybox
        let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nexec /bin/sh\n".to_vec();
        let entries = vec![
            ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
            ("bin/sh".to_string(), busybox_data, 0o100755u32),
            ("init".to_string(), init_script, 0o100755u32),
        ];
        let cpio = build_cpio(&entries);
        let initrd_start = 0x4400_0000u64;
        let initrd_end = initrd_start + cpio.len() as u64;

        let dtb = build_dtb(
            0x4000_0000, 0x4000_0000,
            Some(initrd_start), Some(initrd_end),
            Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"),
        );

        load_initrd(&mut machine.bus, initrd_start, &cpio);
        load_dtb(&mut machine.bus, dtb_addr, &dtb);

        Ok(BootContext {
            machine,
            dtb_addr,
            entry_pc: entry,
            efi_stub_done: false,
            pages_bump: 0x4800_0000u64,
            history: Vec::new(),
        })
    }

    /// Run the EFI stub phase.
    pub fn run_efi_phase(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        let cpu = &mut self.machine.cpus[0];

        for _ in 0..max_steps {
            // Detect EFI stub completion: PC enters kernel VA space
            if !self.efi_stub_done && cpu.regs.pc >= 0xffff800000000000 {
                self.efi_stub_done = true;
                break;
            }
            // Detect return to trampoline (EFI stub exited)
            if !self.efi_stub_done && cpu.regs.pc == 0x43EFE000 {
                self.efi_stub_done = true;
                cpu.regs.pc = 0xffff800080080000; // kernel text entry
                cpu.regs.set_x(0, self.dtb_addr);
                cpu.regs.set_x(1, 0);
                cpu.regs.set_x(2, 0);
                cpu.regs.set_x(3, 0);
                break;
            }

            // EFI service traps
            if cpu.regs.pc == 0x8000_CE00 {
                // CopyMem
                let dest = cpu.regs.x(0);
                let src = cpu.regs.x(1);
                let len = cpu.regs.x(2);
                if len > 0x0400_0000 { break; }
                for i in 0..len {
                    if let Some(val) = self.machine.bus.mem.read(src + i, 1) {
                        self.machine.bus.mem.write(dest + i, 1, val);
                    }
                }
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x8000_D000 {
                // SetMem
                let buf = cpu.regs.x(0);
                let size = cpu.regs.x(1);
                let val = cpu.regs.x(2);
                if size > 0x0400_0000 { break; }
                for i in 0..size { self.machine.bus.mem.write(buf + i, 1, val); }
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x8000_D200 {
                // AllocatePages
                let pages = cpu.regs.x(2);
                let ptr_memory = cpu.regs.x(3);
                let allocated = (self.pages_bump + 4095) & !4095;
                self.pages_bump = allocated + pages * 4096;
                self.machine.bus.write(ptr_memory, 8, allocated);
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x8000_D400 {
                // FreePages
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x400b6e80 {
                // Fast-forward cache invalidation loop
                cpu.regs.set_x(2, cpu.regs.x(3));
                cpu.pstate.set_nzcv(false, true, true, false);
                cpu.regs.pc = 0x400b6e90;
                continue;
            }
            if cpu.regs.pc == 0x400b6eb8 {
                // Fast-forward instruction cache invalidation loop
                cpu.regs.set_x(3, cpu.regs.x(1));
                cpu.pstate.set_nzcv(false, true, true, false);
                cpu.regs.pc = 0x400b6ec8;
                continue;
            }

            // Normal instruction execution for EFI phase
            let pa = match translate(&cpu.sys, &mut cpu.tlb, &self.machine.bus.mem, cpu.regs.pc) {
                Ok(pa) => pa,
                Err(_) => break,
            };
            let raw = match self.machine.bus.mem.read(pa, 4) {
                Some(v) => v as u32,
                None => break,
            };
            if let Some(instr) = decode(raw) {
                self.history.push((cpu.regs.pc, raw, instr.op));
                if self.history.len() > 100 { self.history.remove(0); }
                if let Err(_) = execute(cpu, &mut self.machine.bus, instr) {
                    break;
                }
            } else {
                break;
            }
            steps += 1;
        }

        steps
    }

    /// Run the kernel phase using the multi-core machine.
    /// All cores participate in round-robin execution.
    pub fn run_kernel_phase(&mut self, max_steps: usize) -> usize {
        self.machine.run(max_steps)
    }

    /// Get UART output string.
    pub fn uart_output(&self) -> String {
        self.machine.bus.uart.output_string()
    }

    /// Get total steps across all phases.
    pub fn total_steps(&self) -> u64 { self.machine.total_steps }

    /// Get PC of core 0.
    pub fn pc(&self) -> u64 { self.machine.cpus[0].regs.pc }
}

/// Build identity + kernel page tables for MMU.
fn setup_boot_page_tables(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    let ttbr1_l0 = 0x8010_0000u64;
    let ttbr1_l1 = 0x8010_1000u64;
    let ttbr1_l2 = 0x8010_2000u64;
    let ttbr1_l3_base = 0x8010_3000u64;
    let num_l3_tables = 96usize;
    let ttbr0_l0 = 0x8017_3000u64;
    let ttbr0_l1 = 0x8017_4000u64;

    let l1_block = |pa: u64| -> u64 { pa | (1 << 10) | 0b01 };
    let l3_page = |pa: u64| -> u64 { pa | (1 << 10) | 0b11 };

    // TTBR0: identity map first 4 GB
    bus.write(ttbr0_l0 + 0 * 8, 8, (ttbr0_l1 & 0x0000_FFFF_FFFF_F000) | 0b11);
    for i in 0..4 {
        bus.write(ttbr0_l1 + i * 8, 8, l1_block(i * 0x4000_0000));
    }

    // TTBR1: map kernel VA -> PA
    bus.write(ttbr1_l0 + 256 * 8, 8, (ttbr1_l1 & 0x0000_FFFF_FFFF_F000) | 0b11);
    bus.write(ttbr1_l1 + 2 * 8, 8, (ttbr1_l2 & 0x0000_FFFF_FFFF_F000) | 0b11);
    for tbl in 0..num_l3_tables {
        let l3 = ttbr1_l3_base + (tbl as u64) * 0x1000;
        bus.write(ttbr1_l2 + (tbl as u64) * 8, 8, (l3 & 0x0000_FFFF_FFFF_F000) | 0b11);
        for i in 0..512 {
            let va_offset = (tbl as u64) * 0x20_0000 + (i as u64) * 0x1000;
            bus.write(l3 + i * 8, 8, l3_page(0x4800_0000 + va_offset));
        }
    }

    cpu.sys.ttbr0_el1 = ttbr0_l0;
    cpu.sys.ttbr1_el1 = ttbr1_l0;
    cpu.sys.tcr_el1 = (16 << 16) | 16;
    cpu.sys.mair_el1 = 0xFF;
    cpu.sys.sctlr_el1 = 1;
}
