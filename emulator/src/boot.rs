//! Kernel boot pipeline — loads an ARM64 Linux kernel Image and boots it to a shell.
//!
//! The boot flow has two phases:
//!   1. **EFI stub phase** — runs the kernel's built-in EFI stub (PE/COFF entry point).
//!      The stub calls UEFI services to discover memory and devices.
//!   2. **Kernel phase** — the real kernel entry point, running with MMU enabled.
//!
//! For a beginner: think of the EFI stub as a "bootstrap loader" that hands
//! control to the actual Linux kernel after setting up a minimal environment.

use crate::arm64::{Armv8Cpu, Machine, decode, execute, translate, Opcode};
use crate::bus::SystemBus;
use crate::constants::*;
use crate::efi::setup_efi_tables;
use crate::dtb::{build_dtb, load_dtb};
use crate::initrd::{build_cpio, load_initrd};

/// Holds everything needed to boot and run a Linux kernel.
pub struct BootContext {
    pub machine: Machine,
    pub dtb_addr: u64,
    pub entry_pc: u64,
    efi_stub_done: bool,
    pages_bump: u64,
    history: Vec<(u64, u32, Opcode)>,
}

impl BootContext {
    /// Create a boot context from a raw kernel image (already decompressed).
    ///
    /// This:
    ///   1. Copies the kernel into RAM at KERNEL_LOAD_ADDR
    ///   2. Sets up UEFI firmware tables (SystemTable, BootServices, etc.)
    ///   3. Builds a minimal initrd (busybox)
    ///   4. Creates a Device Tree Blob (DTB) describing the virtual hardware
    ///   5. Configures boot page tables for the MMU
    ///   6. Prepares CPU registers (X0=handle, X1=system_table, SP, LR)
    pub fn new(kernel_image: &[u8], num_cores: usize) -> Result<Self, String> {
        let mut machine = Machine::new(num_cores);

        // Copy kernel image into RAM
        for (i, &byte) in kernel_image.iter().enumerate() {
            machine.bus.write(KERNEL_LOAD_ADDR + i as u64, 1, byte as u64);
        }

        // Parse kernel header at offset 16:8 → image_size (little-endian u64)
        let image_size = read_kernel_image_size(kernel_image);

        // Patch the in-memory PE header: set ImageBase to our actual load
        // address.  The kernel Image has image_base=0x0 and NO .reloc section,
        // so the EFI stub's relocation fails with EFI_LOAD_ERROR.  By setting
        // ImageBase = KERNEL_LOAD_ADDR, the delta is zero and relocation is
        // skipped.
        // PE optional header offset: PE_sig(0x40) + COFF(20) = 0x58
        // ImageBase is at offset 24 within the optional header = 0x58+24 = 0x70
        machine.bus.write(KERNEL_LOAD_ADDR + 0x70, 8, KERNEL_LOAD_ADDR);

        // Patch the .reloc data directory to point to a minimal valid block
        // so efi_pe_relocate_kernel finds relocations and succeeds.
        // PE data directories start at optional_header + 0x70 (= 0xC8 from base)
        // Entry 5 (.reloc) is at dd_start + 5*8 = 0xC8 + 0x28 = 0xF0
        let dd_start: u64 = 0xC8;
        let reloc_entry = KERNEL_LOAD_ADDR + dd_start + 5 * 8;
        // Write a fake .reloc block just past the loaded image
        let fake_reloc = KERNEL_LOAD_ADDR + image_size;
        // PageRVA=0, BlockSize=8 (valid block, no actual entries)
        machine.bus.write(fake_reloc, 4, 0);      // PageRVA = 0
        machine.bus.write(fake_reloc + 4, 4, 8);  // BlockSize = 8
        // Set .reloc data directory to point to our fake block
        // RVA is relative to ImageBase (which we patched to KERNEL_LOAD_ADDR)
        let reloc_rva = fake_reloc.wrapping_sub(KERNEL_LOAD_ADDR);
        machine.bus.write(reloc_entry, 4, image_size as u32 as u64);  // RVA = image_size
        machine.bus.write(reloc_entry + 4, 4, 8);                      // Size = 8

        // Set up EFI firmware tables
        let (handle, system_table) = setup_efi_tables(
            &mut machine.bus, KERNEL_LOAD_ADDR, image_size, DTB_BASE,
        );

        // Configure core 0 as the boot CPU
        let cpu0 = &mut machine.cpus[0];
        cpu0.regs.set_x(0, handle);        // X0 = EFI image handle
        cpu0.regs.set_x(1, system_table);  // X1 = EFI SystemTable pointer
        cpu0.regs.sp = BOOT_STACK_POINTER;

        // Plant a RET instruction at the return trampoline; set LR to point there
        // so the EFI stub's outermost function can return cleanly if it needs to.
        machine.bus.write(RETURN_TRAMPOLINE_ADDR, 4, INSTR_RET as u64);
        cpu0.regs.set_x(LINK_REGISTER_INDEX, RETURN_TRAMPOLINE_ADDR);

        // Build boot page tables (identity map + kernel VA → PA mapping)
        setup_boot_page_tables(cpu0, &mut machine.bus);
        // Enable the MMU with identity mapping so the EFI stub runs in 1:1 PA=VA
        cpu0.sys.sctlr_el1 = SCTLR_MMU_ENABLE;

        // Read PE entry_RVA from the loaded kernel header instead of using
        // a hardcoded constant (which only matches one specific kernel).
        // PE optional header offset: PE_sig(0x40) + COFF(20) + entry_rva(16) = 0x68
        let pe_entry_rva = read_pe_entry_rva(&mut machine.bus);
        let entry = KERNEL_LOAD_ADDR + pe_entry_rva;
        cpu0.regs.pc = entry;

        // Build a minimal initrd (busybox + init script)
        let initrd = build_minimal_initrd();
        let initrd_end = INITRD_BASE + initrd.len() as u64;

        // Build Device Tree Blob
        let dtb = build_dtb(
            RAM_BASE,
            RAM_SIZE,
            Some(INITRD_BASE),
            Some(initrd_end),
            Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"),
        );

        load_initrd(&mut machine.bus, INITRD_BASE, &initrd);
        load_dtb(&mut machine.bus, DTB_BASE, &dtb);

        Ok(BootContext {
            machine,
            dtb_addr: DTB_BASE,
            entry_pc: entry,
            efi_stub_done: false,
            pages_bump: PAGE_ALLOCATOR_BASE,
            history: Vec::new(),
        })
    }

    /// Run the EFI stub phase — up to `max_steps` instructions.
    ///
    /// The PE entry at 0x41E27EE0 runs the EFI stub.  When it finishes,
    /// it RETs through X30 to our trampoline (0x43EFE000) with the kernel
    /// entry address in X0.  We detect this handoff and jump there.
    pub fn run_efi_phase(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        let cpu = &mut self.machine.cpus[0];

        for _ in 0..max_steps {
            // Detect handoff: PE entry returned to our trampoline with
            // kernel entry address in X0.
            if !self.efi_stub_done && cpu.regs.pc == RETURN_TRAMPOLINE_ADDR {
                self.efi_stub_done = true;
                // The PE entry function RETs with whatever efi_main returned.
                // For this kernel (no .reloc), efi_main always fails with
                // EFI_LOAD_ERROR.  Just enter the kernel at KERNEL_LOAD with
                // MMU off and identity map.
                let _retval = cpu.regs.x(0);
                eprintln!("EFI phase complete (X0=0x{:x}), entering kernel at 0x{:x}", _retval, KERNEL_LOAD_ADDR);
                cpu.sys.sctlr_el1 = 0; // disable MMU so physical addresses work
                cpu.regs.pc = KERNEL_LOAD_ADDR;
                cpu.regs.set_x(0, self.dtb_addr);
                cpu.regs.set_x(1, 0);
                cpu.regs.set_x(2, 0);
                cpu.regs.set_x(3, 0);
                break;
            }

            // ── EFI service traps (PC-based dispatch) ──
            if handle_efi_service_trap(cpu, &mut self.machine.bus, &mut self.pages_bump) {
                steps += 1;
                continue;
            }

            // ── Fast-forward cache maintenance loops ──
            if handle_cache_loop_fast_forward(cpu) {
                continue;
            }

            // ── Normal instruction execution ──
            let pa = match translate(&cpu.sys, &mut cpu.tlb, &self.machine.bus.mem, cpu.regs.pc) {
                Ok(pa) => pa,
                Err(_) => { cpu.regs.pc += INSTRUCTION_SIZE; steps += 1; continue; }
            };
            let raw = match self.machine.bus.mem.read(pa, 4) {
                Some(v) => v as u32,
                None => { cpu.regs.pc += INSTRUCTION_SIZE; steps += 1; continue; }
            };
            if let Some(instr) = decode(raw) {
                self.history.push((cpu.regs.pc, raw, instr.op));
                if self.history.len() > INSTR_HISTORY_SIZE { self.history.remove(0); }
                if let Err(_) = execute(cpu, &mut self.machine.bus, instr) {
                    cpu.regs.pc += INSTRUCTION_SIZE;
                }
            } else {
                cpu.regs.pc += INSTRUCTION_SIZE;
            }
            steps += 1;
        }

        steps
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

/// Read the PE/COFF entry point RVA from the loaded kernel image in memory.
fn read_pe_entry_rva(bus: &crate::bus::SystemBus) -> u64 {
    // PE signature at offset 0x40 from kernel start
    // COFF header: 20 bytes → optional header starts at 0x40+4+20 = 0x58
    // Entry point RVA at optional_header + 16 = 0x58 + 16 = 0x68
    bus.mem.read(KERNEL_LOAD_ADDR + 0x68, 4).unwrap_or(0) as u64
}

fn build_minimal_initrd() -> Vec<u8> {
    let busybox_data = vec![0u8; 100]; // dummy busybox — 100 bytes of zeros
    let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nexec /bin/sh\n".to_vec();
    let entries = vec![
        ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
        ("bin/sh".to_string(), busybox_data, 0o100755u32),
        ("init".to_string(), init_script, 0o100755u32),
    ];
    build_cpio(&entries)
}

fn setup_boot_page_tables(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    // Helper: encode an L1 block descriptor (1 GiB) — bit 10=AF, [1:0]=01=block
    let l1_block = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_BLOCK };
    // Helper: encode an L3 page descriptor (4 KiB) — bit 10=AF, [1:0]=11=page
    let l3_page  = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_VALID };

    // ── TTBR0: identity-map the first 4 GiB ──
    // 4 × 1 GiB blocks cover the entire low + RAM regions
    bus.write(BOOT_TTBR0_L0, 8, (BOOT_TTBR0_L1 & DESC_ADDR_MASK) | DESC_VALID);
    for i in 0..IDENTITY_MAP_BLOCKS {
        bus.write(BOOT_TTBR0_L1 + i as u64 * 8, 8, l1_block(i as u64 * L1_BLOCK_SIZE));
    }

    // ── TTBR1: map kernel VA → physical PA ──
    // L0 entry at index 256 (= kernel-space starts at VA bit 47)
    bus.write(BOOT_TTBR1_L0 + 256 * 8, 8, (BOOT_TTBR1_L1 & DESC_ADDR_MASK) | DESC_VALID);
    // L1 entries at index 0 and 2 — cover different kernel VA layouts
    bus.write(BOOT_TTBR1_L1 + 0 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);
    bus.write(BOOT_TTBR1_L1 + 2 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);
    // L2 → L3 for each 2 MiB region
    for tbl in 0..BOOT_TTBR1_L3_COUNT {
        let l3_table_addr = BOOT_TTBR1_L3_BASE + (tbl as u64) * PAGE_SIZE;
        bus.write(BOOT_TTBR1_L2 + (tbl as u64) * 8, 8, (l3_table_addr & DESC_ADDR_MASK) | DESC_VALID);
        // Fill L3 with 4 KiB page entries
        for i in 0..PT_ENTRIES as usize {
            let va_offset = (tbl as u64) * L2_BLOCK_SIZE + (i as u64) * PAGE_SIZE;
            bus.write(l3_table_addr + i as u64 * 8, 8, l3_page(RAM_BASE + va_offset));
        }
    }

    // Configure MMU registers
    cpu.sys.ttbr0_el1 = BOOT_TTBR0_L0;
    cpu.sys.ttbr1_el1 = BOOT_TTBR1_L0;
    cpu.sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16; // 48-bit VA space
    cpu.sys.mair_el1 = MAIR_EL1_DEFAULT;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE; // enable the MMU
}

// ── EFI stub trap handlers ──

/// Returns true if PC matched an EFI trap address and was handled.
fn handle_efi_service_trap(cpu: &mut Armv8Cpu, bus: &mut SystemBus, pages_bump: &mut u64) -> bool {
    match cpu.regs.pc {
        EFI_TRAP_COPYMEM => {
            // CopyMem(Dest=X0, Src=X1, Length=X2)
            let dest = cpu.regs.x(0);
            let src  = cpu.regs.x(1);
            let len  = cpu.regs.x(2);
            if len > EFI_MAX_COPY_SIZE { return false; }
            for i in 0..len {
                if let Some(val) = bus.mem.read(src + i, 1) {
                    bus.mem.write(dest + i, 1, val);
                }
            }
            cpu.regs.set_x(0, EFI_SUCCESS);
            cpu.regs.pc = cpu.regs.x(LINK_REGISTER_INDEX);
            true
        }
        EFI_TRAP_SETMEM => {
            // SetMem(Buffer=X0, Size=X1, Value=X2)
            let buf  = cpu.regs.x(0);
            let size = cpu.regs.x(1);
            let val  = cpu.regs.x(2);
            if size > EFI_MAX_COPY_SIZE { return false; }
            for i in 0..size { bus.mem.write(buf + i, 1, val); }
            cpu.regs.set_x(0, EFI_SUCCESS);
            cpu.regs.pc = cpu.regs.x(LINK_REGISTER_INDEX);
            true
        }
        _ => false,
    }
}

/// Returns true if the current PC is a known cache-invalidation loop (fast-forward it).
fn handle_cache_loop_fast_forward(cpu: &mut Armv8Cpu) -> bool {
    match cpu.regs.pc {
        CACHE_INV_LOOP_ENTRY => {
            // DC CIVAC loop: set counter to range end so SUB/CMP finishes
            cpu.regs.set_x(2, cpu.regs.x(3));
            cpu.pstate.set_nzcv(false, true, true, false); // N=0, Z=1, C=1, V=0 → EQ+CS
            cpu.regs.pc = CACHE_INV_LOOP_EXIT;
            true
        }
        I_CACHE_INV_LOOP_ENTRY => {
            // IC IVAU loop
            cpu.regs.set_x(3, cpu.regs.x(1));
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = I_CACHE_INV_LOOP_EXIT;
            true
        }
        _ => false,
    }
}
