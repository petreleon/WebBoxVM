use super::*;

pub(super) fn print_boot_summary(cpu: &Armv8Cpu, bus: &SystemBus, steps: u64) {
    println!(
        "Executed {} instructions, X0=0x{:016x}",
        steps,
        cpu.regs.x(0)
    );
    println!(
        "  Final: PC=0x{:016x} SP=0x{:016x}",
        cpu.regs.pc, cpu.regs.sp
    );
    println!("  UART Output: {:?}", bus.uart.output_string());
}

pub(super) fn scan_fixmap_for_uart(cpu: &mut Armv8Cpu, bus: &SystemBus) {
    println!("Scanning fixmap for UART OA...");
    let fixmap_end = 0xFFFFFBFFFE000000u64;
    for slot in 0..64u64 {
        let fixmap_va = fixmap_end - (slot + 1) * 0x1000;
        if let Ok(pa) = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, fixmap_va) {
            if (0x08000000..0x0A000000).contains(&pa) {
                println!("  FIXMAP VA={:#018x} -> PA={:#018x}", fixmap_va, pa);
                let l0_idx = (fixmap_va >> 39) & 0x1FF;
                let l1_idx = (fixmap_va >> 30) & 0x1FF;
                let l2_idx = (fixmap_va >> 21) & 0x1FF;
                let l3_idx = (fixmap_va >> 12) & 0x1FF;
                println!(
                    "    indices: L0[{}] L1[{}] L2[{}] L3[{}]",
                    l0_idx, l1_idx, l2_idx, l3_idx
                );
            }
        }
    }
}

pub(super) fn scan_printk_log(bus: &SystemBus) {
    println!("Scanning RAM for printk log_buf...");
    let ram_start = 0x4008_0000u64;
    let ram_end = 0x4700_0000u64;
    let pattern = b"Linux version";
    let mut found_addresses = Vec::new();
    let mut addr = ram_start;

    while addr < ram_end - 16 {
        if pattern
            .iter()
            .enumerate()
            .all(|(i, byte)| bus.mem.read(addr + i as u64, 1) == Some(*byte as u64))
        {
            found_addresses.push(addr);
            addr += pattern.len() as u64;
        } else {
            addr += 1;
        }
    }

    println!(
        "Found 'Linux version' pattern at {} addresses: {:?}",
        found_addresses.len(),
        found_addresses
    );
    for &log_addr in &found_addresses {
        extract_ascii_log(bus, log_addr);
    }
}

fn extract_ascii_log(bus: &SystemBus, log_addr: u64) {
    println!("Extracting ASCII logs around {:#x}:", log_addr);
    let start_addr = log_addr.saturating_sub(1024);
    let mut s = String::new();
    let mut current_seq = String::new();
    for offset in 0..65536 {
        if let Some(b) = bus.mem.read(start_addr + offset, 1) {
            push_ascii_or_flush(b as u8, &mut current_seq, &mut s);
        } else {
            flush_ascii(&mut current_seq, &mut s);
        }
    }
    println!("--- EXTRACTED BOOT LOG ---");
    println!("{}", s);
    println!("--------------------------");
}

fn push_ascii_or_flush(byte: u8, current_seq: &mut String, out: &mut String) {
    if (32..=126).contains(&byte) || byte == 10 || byte == 13 {
        current_seq.push(byte as char);
    } else {
        flush_ascii(current_seq, out);
    }
}

fn flush_ascii(current_seq: &mut String, out: &mut String) {
    if current_seq.len() >= 4 {
        out.push_str(current_seq);
        out.push('\n');
    }
    current_seq.clear();
}
