use super::*;
use crate::arm64::{Armv8Cpu, decode, execute};
use crate::arm64::mmu::translate;
use crate::arm64::opcodes::Opcode;
use crate::bus::SystemBus;

/// Build identity + kernel page tables and enable the MMU.
/// Page tables are placed in the EFI memory region (0x8000_0000+)
/// so the EFI allocator does not overwrite them.
fn setup_boot_page_tables(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    // 48-bit VA (T0SZ=16, T1SZ=16) — matches the Debian kernel config.
    // Page tables placed well above EFI structures (0x8000_0000–0x8000_FFFF)
    // to avoid overwriting Handle, SystemTable, RuntimeServices, BootServices.
    let ttbr1_l0 = 0x8010_0000u64; // TTBR1 L0
    let ttbr1_l1 = 0x8010_1000u64; // TTBR1 L1
    let ttbr1_l2 = 0x8010_2000u64; // TTBR1 L2
    // L3 tables placed contiguously; each covers 2 MB.
    // 96 tables = 192 MB coverage, covering physical 0x4008_0000 to 0x4c08_0000.
    // This fully covers the kernel image, initrd, DTB, and all EFI-allocated pools.
    let ttbr1_l3_base = 0x8010_3000u64;
    let num_l3_tables = 96usize;
    let ttbr0_l0 = 0x8017_3000u64; // TTBR0 L0 (moved past 96 L3 tables which end at 0x8016_3000)
    let ttbr0_l1 = 0x8017_4000u64; // TTBR0 L1
 
    let l1_block = |pa: u64| -> u64 { pa | (1 << 10) | 0b01 };
    let l3_page = |pa: u64| -> u64 { pa | (1 << 10) | 0b11 };
 
    // ── TTBR0: identity map first 4 GB with 1 GB L1 blocks ──
    bus.write(ttbr0_l0 + 0 * 8, 8, (ttbr0_l1 & 0x0000_FFFF_FFFF_F000) | 0b11);
    for i in 0..4 {
        bus.write(ttbr0_l1 + i * 8, 8, l1_block(i * 0x4000_0000));
    }
 
    // ── TTBR1: map kernel VA 0xffff800080000000 -> PA 0x40080000 via L3 pages ──
    // L0 entry 256 covers bits [47:39] = 256
    bus.write(ttbr1_l0 + 256 * 8, 8, (ttbr1_l1 & 0x0000_FFFF_FFFF_F000) | 0b11);
    // L1 entry 2 covers bits [38:30] = 2 for 0xffff800080000000
    bus.write(ttbr1_l1 + 2 * 8, 8, (ttbr1_l2 & 0x0000_FFFF_FFFF_F000) | 0b11);
    // L2 entries 0..95 each point to an L3 table covering 2 MB.
    for tbl in 0..num_l3_tables {
        let l3 = ttbr1_l3_base + (tbl as u64) * 0x1000;
        bus.write(ttbr1_l2 + (tbl as u64) * 8, 8, (l3 & 0x0000_FFFF_FFFF_F000) | 0b11);
        for i in 0..512 {
            let va_offset = (tbl as u64) * 0x20_0000 + (i as u64) * 0x1000;
            bus.write(l3 + i * 8, 8, l3_page(0x4800_0000 + va_offset));
        }
    }
 
    // ── System registers ──
    cpu.sys.ttbr0_el1 = ttbr0_l0;
    cpu.sys.ttbr1_el1 = ttbr1_l0;
    cpu.sys.tcr_el1 = (16 << 16) | 16; // 48-bit VA for both TTBR0 and TTBR1
    cpu.sys.mair_el1 = 0xFF;            // Attr0 = normal WBWA
    cpu.sys.sctlr_el1 = 1;              // Enable MMU
}

// ─────────────────────────────────────────────────────────────────────────────
// Main test: real kernel boot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue() {
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;
    use crate::initrd::{build_cpio, load_initrd};
    use crate::dtb::{build_dtb, load_dtb};

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let dtb_addr = 0x4700_0000u64;
    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000, dtb_addr);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43F0_0000;

    bus.write(0x43EFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43EFE000);

    setup_boot_page_tables(&mut cpu, &mut bus);

    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let mut steps = 0u64;
    let mut last_pc = cpu.regs.pc;
    let mut efi_stub_done = false;

    let busybox_data = std::fs::read("/tmp/busybox").unwrap_or_else(|_| vec![0u8; 100]);
    let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nmount -t sysfs sysfs /sys\nexec /bin/sh\n".to_vec();
    let entries = vec![
        ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
        ("bin/sh".to_string(), busybox_data.clone(), 0o100755u32),
        ("init".to_string(), init_script, 0o100755u32),
        ("proc".to_string(), Vec::new(), 0o040755u32),
        ("sys".to_string(), Vec::new(), 0o040755u32),
    ];
    let cpio = build_cpio(&entries);
    let initrd_start = 0x4400_0000u64;
    let initrd_end = initrd_start + cpio.len() as u64;

    let dtb = build_dtb(
        0x4000_0000, 0x4000_0000,
        Some(initrd_start), Some(initrd_end),
        Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"),
    );
    

    load_initrd(&mut bus, initrd_start, &cpio);
    load_dtb(&mut bus, dtb_addr, &dtb);

    let mut pages_bump = 0x4800_0000u64;
    let mut history = std::collections::VecDeque::with_capacity(105);
    let mut decode_cache = crate::arm64::DecodeCache::new();

    for _ in 0..20_000_000usize {
        if steps % 2_000_000 == 0 {
            eprintln!("PROGRESS: {:.1}M steps, cache {}/{}", steps as f64 / 1_000_000.0, decode_cache.hits, decode_cache.misses);
        }
        if !efi_stub_done && cpu.regs.pc == 0 {
            println!("EFI Stub completed in {} steps. Transitioning to main kernel...", steps);
            efi_stub_done = true;
            cpu.regs.pc = 0xffff800080080000;
            // cpu.regs.set_x(0, dtb_addr); // DO NOT OVERWRITE! Keep the updated DTB address returned by the EFI stub in X0!
            cpu.regs.set_x(1, 0);
            cpu.regs.set_x(2, 0);
            cpu.regs.set_x(3, 0);
            last_pc = cpu.regs.pc;
            continue;
        }
        if cpu.regs.pc == 0x8000_CE00 {
            // CopyMem(Dest, Src, Length)
            let dest = cpu.regs.x(0);
            let src = cpu.regs.x(1);
            let mut len = cpu.regs.x(2);
            if len > 0x0400_0000 {
                println!("WARNING: CopyMem with huge len = {:#x}, capping to 0", len);
                len = 0;
            }
            if dest > src && src + len > dest {
                // Copy right-to-left
                for i in (0..len).rev() {
                    let val = bus.mem.read(src + i, 1).unwrap_or(0);
                    bus.mem.write(dest + i, 1, val);
                }
            } else {
                // Copy left-to-right
                for i in 0..len {
                    let val = bus.mem.read(src + i, 1).unwrap_or(0);
                    bus.mem.write(dest + i, 1, val);
                }
            }
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D000 {
            // SetMem(Buffer, Size, Value)
            let buf = cpu.regs.x(0);
            let mut size = cpu.regs.x(1);
            let val = cpu.regs.x(2);
            if size > 0x0400_0000 {
                println!("WARNING: SetMem with huge size = {:#x}, capping to 0", size);
                size = 0;
            }
            for i in 0..size {
                bus.mem.write(buf + i, 1, val);
            }
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D200 {
            // AllocatePages(Type, MemoryType, Pages, &Memory)
            let pages = cpu.regs.x(2);
            let ptr_memory = cpu.regs.x(3);
            let allocated = (pages_bump + 4095) & !4095;
            pages_bump = allocated + pages * 4096;
            bus.write(ptr_memory, 8, allocated);
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D400 {
            // FreePages(Memory, Pages)
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x400b6e80 {
            // Fast-forward cache invalidation loop
            let x3 = cpu.regs.x(3);
            cpu.regs.set_x(2, x3);
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = 0x400b6e90;
            continue;
        }
        if cpu.regs.pc == 0x400b6eb8 {
            // Fast-forward instruction cache invalidation loop
            let x1 = cpu.regs.x(1);
            cpu.regs.set_x(3, x1);
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = 0x400b6ec8;
            continue;
        }
        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(addr) => addr,
            Err(_) => {
                println!("Translation fault at step {} PC=0x{:016x}", steps, cpu.regs.pc);
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!("Memory fault at step {} PC=0x{:016x} PA=0x{:016x}", steps, cpu.regs.pc, pa);
                break;
            }
        };
        // Use decode cache: fetches pre-decoded Instr by PA
        let maybe_instr = decode_cache.fetch(&bus.mem, pa);
        if let Some(instr) = maybe_instr {
            history.push_back((cpu.regs.pc, raw, instr));
            if history.len() > 100 {
                history.pop_front();
            }
            if instr.op == Opcode::Brk {
                println!("--- LAST 100 EXECUTED INSTRUCTIONS ---");
                for (hist_pc, hist_raw, hist_instr) in &history {
                    println!("  0x{:016x}: {:08x} {:?}", hist_pc, hist_raw, hist_instr);
                }
                println!("--------------------------------------");
            }
            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!("EXECUTE ERROR at step {} PC=0x{:016x}: {:?}", steps, cpu.regs.pc, e);
                break;
            }
            steps += 1;
            if cpu.regs.pc == last_pc {
                println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("Executed {} instructions, X0=0x{:016x}", steps, cpu.regs.x(0));
    println!("  Final: PC=0x{:016x} SP=0x{:016x}", cpu.regs.pc, cpu.regs.sp);
    println!("  UART Output: {:?}", bus.uart.output_string());

    // RAM Scanner for kernel printk log_buf
    println!("Scanning RAM for printk log_buf...");
    let ram_start = 0x4008_0000u64;
    let ram_end = 0x4700_0000u64;
    let pattern = b"Linux version";
    let mut found_addresses = Vec::new();
    let mut addr = ram_start;
    while addr < ram_end - 16 {
        let mut matched = true;
        for i in 0..pattern.len() {
            if bus.mem.read(addr + i as u64, 1) != Some(pattern[i] as u64) {
                matched = false;
                break;
            }
        }
        if matched {
            found_addresses.push(addr);
            addr += pattern.len() as u64;
        } else {
            addr += 1;
        }
    }

    println!("Found 'Linux version' pattern at {} addresses: {:?}", found_addresses.len(), found_addresses);
    for &log_addr in &found_addresses {
        println!("Extracting ASCII logs around {:#x}:", log_addr);
        let start_addr = if log_addr > 1024 { log_addr - 1024 } else { log_addr };
        let mut s = String::new();
        let mut current_seq = String::new();
        for offset in 0..65536 {
            if let Some(b) = bus.mem.read(start_addr + offset, 1) {
                let b = b as u8;
                if (b >= 32 && b <= 126) || b == 10 || b == 13 {
                    current_seq.push(b as char);
                } else {
                    if current_seq.len() >= 4 {
                        s.push_str(&current_seq);
                        s.push('\n');
                    }
                    current_seq.clear();
                }
            } else {
                if current_seq.len() >= 4 {
                    s.push_str(&current_seq);
                    s.push('\n');
                }
                current_seq.clear();
            }
        }
        println!("--- EXTRACTED BOOT LOG ---");
        println!("{}", s);
        println!("--------------------------");
    }

    panic!("Force panic to see stdout! Executed {} instructions", steps);
}

// ─────────────────────────────────────────────────────────────────────────────
// Trace test: smart EFI call logger
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue_trace() {
    // Logs EFI service call/return pairs; writes full trace to /tmp/kernel_trace.txt.
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;
    use crate::initrd::{build_cpio, load_initrd};
    use crate::dtb::{build_dtb, load_dtb};
    use crate::efi::layout::{EFI_LARGE_CODE, LARGE_CODE_STRIDE, EFI_SERVICE_TRAMPOLINES,
                              TRAMPOLINE_STRIDE, EFI_BOOT_SERVICES, EFI_MEM_BASE};
    use std::io::Write;

    let mut cpu = Armv8Cpu::new();
    cpu.pstate = cpu.pstate.with_el(1);
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let dtb_addr = 0x4700_0000u64;
    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000, dtb_addr);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43F0_0000;
    cpu.regs.set_x(18, st);

    bus.write(0x43EFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43EFE000);

    setup_boot_page_tables(&mut cpu, &mut bus);
    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let busybox_data = std::fs::read("/tmp/busybox").unwrap_or_else(|_| vec![0u8; 100]);
    let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nexec /bin/sh\n".to_vec();
    let entries = vec![
        ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
        ("bin/sh".to_string(), busybox_data.clone(), 0o100755u32),
        ("init".to_string(), init_script, 0o100755u32),
        ("proc".to_string(), Vec::new(), 0o040755u32),
        ("sys".to_string(), Vec::new(), 0o040755u32),
    ];
    let cpio = build_cpio(&entries);
    let initrd_start = 0x4400_0000u64;
    let initrd_end = initrd_start + cpio.len() as u64;
    let dtb = build_dtb(0x4000_0000, 0x4000_0000,
        Some(initrd_start), Some(initrd_end), Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"));
    
    load_initrd(&mut bus, initrd_start, &cpio);
    load_dtb(&mut bus, dtb_addr, &dtb);

    // ── Helpers ──
    let is_efi = |pc: u64| -> bool {
        (pc >= EFI_SERVICE_TRAMPOLINES && pc < EFI_SERVICE_TRAMPOLINES + 512 * TRAMPOLINE_STRIDE)
            || (pc >= EFI_LARGE_CODE && pc < EFI_LARGE_CODE + 16 * LARGE_CODE_STRIDE)
    };

    let bs_offsets: &[(u64, &str)] = &[
        (0x18,"RaiseTPL"),(0x20,"RestoreTPL"),(0x28,"AllocatePages"),(0x30,"FreePages"),
        (0x38,"GetMemoryMap"),(0x40,"AllocatePool"),(0x48,"FreePool"),
        (0x50,"CreateEvent"),(0x58,"SetTimer"),(0x60,"WaitForEvent"),
        (0x68,"SignalEvent"),(0x70,"CloseEvent"),(0x78,"CheckEvent"),
        (0x80,"InstallProtocol"),(0x88,"ReinstallProto"),(0x90,"UninstallProto"),
        (0x98,"HandleProtocol"),(0xA0,"Reserved"),(0xA8,"RegisterProtoNotify"),
        (0xB0,"LocateHandle"),(0xB8,"LocateDevicePath"),(0xC0,"InstallConfigTable"),
        (0xC8,"LoadImage"),(0xD0,"StartImage"),(0xD8,"Exit"),(0xE0,"UnloadImage"),
        (0xE8,"ExitBootServices"),(0xF0,"GetNextMonotonicCount"),
        (0xF8,"Stall"),(0x100,"SetWatchdogTimer"),
        (0x108,"ConnectController"),(0x110,"DisconnectController"),
        (0x118,"OpenProtocol"),(0x120,"CloseProtocol"),
        (0x128,"OpenProtocolInfo"),(0x130,"ProtocolsPerHandle"),
        (0x138,"LocateHandleBuffer"),(0x140,"LocateProtocol"),
        (0x148,"InstallMultipleProtos"),(0x150,"UninstallMultipleProtos"),
        (0x158,"CalculateCrc32"),(0x160,"CopyMem"),(0x168,"SetMem"),(0x170,"CreateEventEx"),
    ];


    let con_out_output_string = bus.mem.read(EFI_MEM_BASE + 0x6000 + 0x08, 8).unwrap_or(0);

    // Build fp->name map before the main loop to avoid borrow conflicts during execute()
    let mut fp_to_name: std::collections::HashMap<u64, String> = std::collections::HashMap::new();
    for &(off, name) in bs_offsets {
        if let Some(fp) = bus.mem.read(EFI_BOOT_SERVICES + off, 8) {
            fp_to_name.insert(fp, name.to_string());
        }
    }
    if con_out_output_string != 0 {
        fp_to_name.insert(con_out_output_string, "ConOut::OutputString".to_string());
    }

    let resolve_efi = |fp: u64| -> String {
        fp_to_name.get(&fp).cloned().unwrap_or_else(|| format!("EFI@{:#x}", fp))
    };


    let efi_status = |s: u64| -> &'static str {
        match s {
            0 => "EFI_SUCCESS",
            0x8000_0000_0000_0001 => "EFI_LOAD_ERROR",
            0x8000_0000_0000_0002 => "EFI_INVALID_PARAM",
            0x8000_0000_0000_0003 => "EFI_UNSUPPORTED",
            0x8000_0000_0000_0005 => "EFI_BUFFER_TOO_SMALL",
            0x8000_0000_0000_000E => "EFI_NOT_FOUND",
            _ => "UNKNOWN_STATUS",
        }
    };

    let mut steps = 0u64;
    let mut last_pc = cpu.regs.pc;
    let mut efi_stub_done = false;
    let mut efi_stack: Vec<(u64, u64, u64, u64, u64, u64)> = Vec::new(); // (caller, entry, x0, x1, x2, x3)
    let mut efi_log: Vec<String> = Vec::new();
    let mut recent: std::collections::VecDeque<String> = std::collections::VecDeque::with_capacity(120);

    let mut trace_file = std::fs::File::create("/tmp/kernel_trace.txt").ok();

    let mut pages_bump = 0x4800_0000u64;

    println!("DTB bytes at {:#x}:", dtb_addr);
    for i in 0..16u64 {
        print!("{:02x} ", bus.mem.read(dtb_addr + i, 1).unwrap_or(0xFF));
    }
    println!();
    
    let st = crate::efi::EFI_SYSTEM_TABLE;
    let config_table_ptr = bus.mem.read(st + 0x70, 8).unwrap_or(0);
    println!("ConfigurationTable pointer: {:#x}", config_table_ptr);
    if config_table_ptr != 0 {
        let g0 = bus.mem.read(config_table_ptr + 0, 8).unwrap_or(0);
        let g1 = bus.mem.read(config_table_ptr + 8, 8).unwrap_or(0);
        let ptr = bus.mem.read(config_table_ptr + 16, 8).unwrap_or(0);
        println!("  GUID: {:#018x} {:#018x} -> table: {:#x}", g0, g1, ptr);
    }

    'main: for _ in 0..60_000usize {
        // Hand-off to main kernel
        if !efi_stub_done && cpu.regs.pc == 0 {
            println!("=== EFI Stub done ({} steps) => main kernel ===", steps);
            efi_stub_done = true;
            cpu.regs.pc = 0xffff800080080000;
            cpu.regs.set_x(0, dtb_addr);
            cpu.regs.set_x(1, 0);
            cpu.regs.set_x(2, 0);
            cpu.regs.set_x(3, 0);
            last_pc = cpu.regs.pc;
            continue;
        }
        if cpu.regs.pc == 0x8000_CE00 {
            // CopyMem(Dest, Src, Length)
            let dest = cpu.regs.x(0);
            let src = cpu.regs.x(1);
            let mut len = cpu.regs.x(2);
            if len > 0x0400_0000 {
                println!("WARNING: CopyMem with huge len = {:#x}, capping to 0", len);
                len = 0;
            }
            if dest > src && src + len > dest {
                // Copy right-to-left
                for i in (0..len).rev() {
                    let val = bus.mem.read(src + i, 1).unwrap_or(0);
                    bus.mem.write(dest + i, 1, val);
                }
            } else {
                // Copy left-to-right
                for i in 0..len {
                    let val = bus.mem.read(src + i, 1).unwrap_or(0);
                    bus.mem.write(dest + i, 1, val);
                }
            }
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D000 {
            // SetMem(Buffer, Size, Value)
            let buf = cpu.regs.x(0);
            let mut size = cpu.regs.x(1);
            let val = cpu.regs.x(2);
            if size > 0x0400_0000 {
                println!("WARNING: SetMem with huge size = {:#x}, capping to 0", size);
                size = 0;
            }
            for i in 0..size {
                bus.mem.write(buf + i, 1, val);
            }
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D200 {
            // AllocatePages(Type, MemoryType, Pages, &Memory)
            let pages = cpu.regs.x(2);
            let ptr_memory = cpu.regs.x(3);
            let allocated = (pages_bump + 4095) & !4095;
            pages_bump = allocated + pages * 4096;
            bus.write(ptr_memory, 8, allocated);
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x8000_D400 {
            // FreePages(Memory, Pages)
            cpu.regs.set_x(0, 0); // EFI_SUCCESS
            cpu.regs.pc = cpu.regs.x(30); // Return to LR
            steps += 1;
            continue;
        }
        if cpu.regs.pc == 0x400b6e80 {
            // Fast-forward cache invalidation loop
            let x3 = cpu.regs.x(3);
            cpu.regs.set_x(2, x3);
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = 0x400b6e90;
            continue;
        }
        if cpu.regs.pc == 0x400b6eb8 {
            // Fast-forward instruction cache invalidation loop
            let x1 = cpu.regs.x(1);
            cpu.regs.set_x(3, x1);
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = 0x400b6ec8;
            continue;
        }

        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(a) => a,
            Err(_) => {
                println!("TRANSLATION FAULT step={} PC={:#016x}", steps, cpu.regs.pc);
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!("MEMORY FAULT step={} PC={:#016x} PA={:#016x}", steps, cpu.regs.pc, pa);
                break;
            }
        };

        let maybe_instr = decode(raw);
        if let Some(ref instr) = maybe_instr {
            // Detect call INTO EFI
            let target: Option<u64> = match instr.op {
                Opcode::Blr => Some(cpu.regs.x(instr.rn)),

                Opcode::Bl  => Some((cpu.regs.pc as i64 + instr.imm as i64) as u64),
                _ => None,
            };
            if let Some(tgt) = target {
                if is_efi(tgt) {
                    let name = resolve_efi(tgt);
                    efi_stack.push((cpu.regs.pc, tgt,
                        cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3)));
                    let mut s = format!("[{:7}] CALL {:<32} caller={:#x} X0={:#x} X1={:#x} X2={:#x} X3={:#x}",
                        steps, name, cpu.regs.pc,
                        cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3));
                    if name == "ConOut::OutputString" {
                        let mut utf16_str = String::new();
                        let mut addr = cpu.regs.x(1);
                        loop {
                            let ch = bus.mem.read(addr, 2).unwrap_or(0);
                            if ch == 0 { break; }
                            if let Some(c) = std::char::from_u32(ch as u32) {
                                utf16_str.push(c);
                            }
                            addr += 2;
                        }
                        s = format!("{} STR={:?}", s, utf16_str);
                    }
                    println!("{}", s);
                    efi_log.push(s);
                }
            }

            // Detect return FROM EFI
            if matches!(instr.op, Opcode::Ret) && is_efi(cpu.regs.pc) {
                if let Some((caller, entry, ax0, ax1, ax2, ax3)) = efi_stack.pop() {
                    let name = resolve_efi(entry);
                    let r0 = cpu.regs.x(0);
                    let ss = efi_status(r0);
                    let s = format!("[{:7}] RET  {:<32} -> {} ({:#x})", steps, name, ss, r0);
                    println!("{}", s);
                    efi_log.push(s);
                    if r0 == 0x8000_0000_0000_0001 {
                        println!("!!! FIRST EFI_LOAD_ERROR: {} (caller={:#x})", name, caller);
                        println!("    Args: X0={:#x} X1={:#x} X2={:#x} X3={:#x}", ax0, ax1, ax2, ax3);
                        if ax1 >= 0x1000 && ax1 < 0x1_0000_0000 {
                            print!("    [X1]= ");
                            for i in 0..16u64 {
                                print!("{:02x} ", bus.mem.read(ax1 + i, 1).unwrap_or(0xff) as u8);
                            }
                            println!();
                        }
                    }
                }
            }
        }

        // Execute
        if let Some(instr) = maybe_instr {
            if let Some(ref mut f) = trace_file {
                let _ = writeln!(f, "{:7} {:#016x} {:?} X0={:#018x} X19={:#018x} SP={:#016x}",
                    steps, cpu.regs.pc, instr.op, cpu.regs.x(0), cpu.regs.x(19), cpu.regs.sp);
            }
            recent.push_back(format!("{:7} {:#016x} {:?}", steps, cpu.regs.pc, instr.op));
            if recent.len() > 120 { recent.pop_front(); }

            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!("EXECUTE ERROR step={} PC={:#016x}: {:?}", steps, cpu.regs.pc, e);
                break 'main;
            }
            steps += 1;
            if !efi_stub_done && cpu.regs.pc == last_pc {
                println!("Stalled PC={:#016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTR step={} PC={:#016x} raw={:#010x}", steps, cpu.regs.pc, raw);
            for l in recent.iter().rev().take(30).collect::<Vec<_>>().iter().rev() { println!("{}", l); }
            break;
        }
    }

    println!("\n=== EFI Call Log ({} calls) ===", efi_log.len());
    for l in &efi_log { println!("{}", l); }
    println!("\n=== Final State ({} steps) ===", steps);
    println!("  PC={:#016x}  SP={:#016x}", cpu.regs.pc, cpu.regs.sp);
    println!("  X0={:#018x}  X1={:#018x}", cpu.regs.x(0), cpu.regs.x(1));
    println!("  X19={:#018x} X20={:#018x} X21={:#018x}", cpu.regs.x(19), cpu.regs.x(20), cpu.regs.x(21));
    println!("  UART: {:?}", bus.uart.output_string());
    println!("  Trace → /tmp/kernel_trace.txt");
    println!("\n--- Last 80 from ring ---");
    for l in recent.iter().rev().take(80).collect::<Vec<_>>().iter().rev() { println!("{}", l); }
}

// ─────────────────────────────────────────────────────────────────────────────
// Synthetic kernel tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn synthetic_kernel_boots_to_uart() {
    use crate::loader::kernel::{load_raw_image, KERNEL_LOAD};
    use std::fs;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let data = fs::read("/tmp/kernel_raw.bin")
        .expect("kernel not found; run build_kernel.sh first");
    load_raw_image(&mut bus, &data);

    cpu.regs.sp = 0x43F0_0000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 50);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert!(bus.uart.output_string().contains("Uncompressing Linux..."), "UART output missing expected message");
}

#[test]
fn synthetic_kernel_reads_initrd_from_dtb() {
    use crate::initrd::{build_cpio, load_initrd};
    use crate::dtb::{build_dtb, load_dtb};
    use crate::loader::kernel::{load_raw_image, KERNEL_LOAD};

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let entries = vec![(
        "init".to_string(),
        b"hello from initrd".to_vec(),
        0o755u32,
    )];
    let cpio = build_cpio(&entries);
    let initrd_start = 0x4200_0000u64;
    let initrd_end = initrd_start + cpio.len() as u64;

    let dtb = build_dtb(
        0x4000_0000, 0x4000_0000,
        Some(initrd_start), Some(initrd_end),
        Some("earlycon console=ttyAMA0"),
    );
    let dtb_addr = 0x4800_0000u64;

    load_initrd(&mut bus, initrd_start, &cpio);
    load_dtb(&mut bus, dtb_addr, &dtb);

    let kernel: Vec<u32> = vec![
        0xD2A12002, // MOVZ X2, #0x0900_0000
        0xD2A84003, // MOVZ X3, #0x4200, LSL #16
        0x39400060, // LDRB W0, [X3]
        0x38000040, // STRB W0, [X2]
        0x39400460, // LDRB W0, [X3, #1]
        0x38000040, // STRB W0, [X2]
        0x14000000, // B .
    ];
    let kernel_bytes: Vec<u8> = kernel.iter().flat_map(|&w| w.to_le_bytes()).collect();
    load_raw_image(&mut bus, &kernel_bytes);

    cpu.regs.set_x(0, dtb_addr);
    cpu.regs.sp = 0x43F0_0000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 20);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    println!("UART output string: {:?}", bus.uart.output_string());

    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert_eq!(bus.uart.output, vec![b'0', b'7'], "Expected first two bytes of cpio magic on UART");
}
