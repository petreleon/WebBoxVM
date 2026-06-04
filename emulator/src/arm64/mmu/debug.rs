use super::*;

#[allow(dead_code)]
pub fn page_table_debug(sys: &SystemRegisters, mem: &PhysicalMemory, va: u64) {
    let ttbr = sys.ttbr1_el1 & DESC_ADDR_MASK;
    let tcr = sys.tcr_el1;
    let t1sz = ((tcr >> TCR_T1SZ_SHIFT) & TCR_T1SZ_MASK) as u8;
    let va_bits = 64u8.saturating_sub(t1sz);

    eprintln!(
        "    TTBR1=0x{:016x}  TCR=0x{:016x}  T1SZ={}  VA_BITS={}",
        ttbr, tcr, t1sz, va_bits
    );
    eprintln!("    VA =0x{:016x}", va);
    debug_l0(mem, va, ttbr);
}

fn debug_l0(mem: &PhysicalMemory, va: u64, ttbr: u64) {
    let l0_idx = ((va >> PT_L0_SHIFT) & 0x1FF) as u64;
    let l0_addr = ttbr + l0_idx * 8;
    let Some(desc) = mem.read(l0_addr, 8) else {
        eprintln!("    L0[{}] at PA=0x{:x} UNREADABLE", l0_idx, l0_addr);
        return;
    };

    let valid = desc & 3;
    eprintln!(
        "    L0[{}] at PA=0x{:x} desc=0x{:016x} valid={}",
        l0_idx, l0_addr, desc, valid
    );
    if valid == DESC_TABLE {
        debug_l1(mem, va, desc & DESC_ADDR_MASK);
    } else if valid == DESC_BLOCK {
        let pa = (desc & 0x0000_FFFF_FFFF_F000) | (va & (L0_BLOCK_SIZE - 1));
        eprintln!("    L0 block -> PA=0x{:016x}", pa);
    } else {
        eprintln!("    L0 INVALID (valid={})", valid);
    }
}

fn debug_l1(mem: &PhysicalMemory, va: u64, base: u64) {
    let idx = ((va >> PT_L1_SHIFT) & 0x1FF) as u64;
    let addr = base + idx * 8;
    let Some(desc) = mem.read(addr, 8) else {
        eprintln!("    L1[{}] at PA=0x{:x} UNREADABLE", idx, addr);
        return;
    };
    let valid = desc & 3;
    eprintln!(
        "    L1[{}] at PA=0x{:x} desc=0x{:016x} valid={}",
        idx, addr, desc, valid
    );
    if valid == DESC_TABLE {
        debug_l2(mem, va, desc & DESC_ADDR_MASK);
    } else if valid == DESC_BLOCK {
        let pa = (desc & 0x0000_FFFF_C000_0000) | (va & (L1_BLOCK_SIZE - 1));
        eprintln!("    L1 block -> PA=0x{:016x}", pa);
    } else {
        eprintln!("    L1 INVALID (valid={})", valid);
    }
}

fn debug_l2(mem: &PhysicalMemory, va: u64, base: u64) {
    let idx = ((va >> PT_L2_SHIFT) & 0x1FF) as u64;
    let addr = base + idx * 8;
    let Some(desc) = mem.read(addr, 8) else {
        eprintln!("    L2[{}] at PA=0x{:x} UNREADABLE", idx, addr);
        return;
    };
    let valid = desc & 3;
    eprintln!(
        "    L2[{}] at PA=0x{:x} desc=0x{:016x} valid={}",
        idx, addr, desc, valid
    );
    if valid == DESC_TABLE {
        debug_l3(mem, va, desc & DESC_ADDR_MASK);
    } else if valid == DESC_BLOCK {
        let pa = (desc & 0x0000_FFFF_FFE0_0000) | (va & (L2_BLOCK_SIZE - 1));
        eprintln!("    L2 block -> PA=0x{:016x}", pa);
    } else {
        eprintln!("    L2 INVALID (valid={})", valid);
    }
}

fn debug_l3(mem: &PhysicalMemory, va: u64, base: u64) {
    let idx = ((va >> PT_L3_SHIFT) & 0x1FF) as u64;
    let addr = base + idx * 8;
    if let Some(desc) = mem.read(addr, 8) {
        let pa = (desc & DESC_ADDR_MASK) | (va & PAGE_OFFSET_MASK);
        eprintln!(
            "    L3[{}] at PA=0x{:x} desc=0x{:016x} -> PA=0x{:016x}",
            idx, addr, desc, pa
        );
    } else {
        eprintln!("    L3[{}] at PA=0x{:x} UNREADABLE", idx, addr);
    }
}
