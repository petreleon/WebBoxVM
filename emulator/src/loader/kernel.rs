//! Load an ARM64 Linux kernel Image into emulator RAM.
//!
//! Linux ARM64 kernels come in two formats:
//!   1. **Raw Image** — a bare binary with a 64-byte header at offset 0
//!   2. **PE/COFF Image** — a Windows-style executable with EFI stub
//!
//! The current code handles both, detecting the PE signature at offset 0x40.

use crate::constants::*;
use crate::bus::SystemBus;
use std::fs;

// Re-export kernel load address for backward compatibility
pub const KERNEL_LOAD: u64 = KERNEL_LOAD_ADDR;

/// Parsed fields from the Linux kernel Image header (first 64 bytes).
struct KernelHeader {
    _code0: u32,
    _code1: u32,
    text_offset: u64,    // offset from load address to kernel entry
    image_size: u64,     // total size the kernel Image occupies
    _flags: u64,
    _res1: u64,
    _res2: u64,
    _magic: u32,         // must be "ARM\x64"
    _res3: u32,
}

/// Load a kernel Image from disk and return the entry-point physical address.
pub fn load_kernel(bus: &mut SystemBus, path: &str) -> Result<u64, &'static str> {
    let data = fs::read(path).map_err(|_| "read failed")?;
    let header = parse_header(&data)?;

    let load_size = if header.image_size > 0 {
        header.image_size as usize
    } else {
        data.len()
    };

    // Copy payload to RAM
    let payload = &data[..load_size.min(data.len())];
    for (i, &byte) in payload.iter().enumerate() {
        bus.write(KERNEL_LOAD_ADDR + i as u64, 1, byte as u64);
    }

    let entry = if is_pe_image(&data) {
        let entry = parse_pe_entry(&data)?;
        let img_size = header.image_size.max(data.len() as u64);
        crate::efi::setup_efi_tables(bus, KERNEL_LOAD_ADDR, img_size, DTB_BASE);
        entry
    } else {
        KERNEL_LOAD_ADDR + header.text_offset
    };

    Ok(entry)
}

/// Copy a raw kernel image into RAM (used for in-memory images, e.g. from wasm-bindgen).
pub fn load_raw_image(bus: &mut SystemBus, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        bus.write(KERNEL_LOAD_ADDR + i as u64, 1, byte as u64);
    }
}

fn parse_header(data: &[u8]) -> Result<KernelHeader, &'static str> {
    if data.len() < 64 {
        return Err("kernel file too small (need >= 64 bytes)");
    }
    let r32 = |o: usize| u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
    let r64 = |o: usize| {
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[o..o + 8]);
        u64::from_le_bytes(b)
    };
    let magic = r32(56);
    if magic != ARM64_KERNEL_MAGIC {
        return Err("bad ARM64 kernel magic (expected \"ARM\\x64\")");
    }
    Ok(KernelHeader {
        _code0: r32(0),
        _code1: r32(4),
        text_offset: r64(8),
        image_size: r64(16),
        _flags: r64(24),
        _res1: r64(32),
        _res2: r64(40),
        _magic: r32(56),
        _res3: r32(60),
    })
}

/// Check if the data contains a PE/COFF signature at offset 0x40.
fn is_pe_image(data: &[u8]) -> bool {
    data.len() > KERNEL_PE_OFFSET + 4 && &data[KERNEL_PE_OFFSET..KERNEL_PE_OFFSET + 4] == PE_SIGNATURE.as_slice()
}

/// Parse the PE optional header to find the entry-point RVA.
fn parse_pe_entry(data: &[u8]) -> Result<u64, &'static str> {
    let opt_start = KERNEL_PE_OFFSET + 24; // PE sig + COFF header = 24 bytes
    if data.len() < opt_start + 20 {
        return Err("PE optional header truncated");
    }
    // Entry RVA is at offset 16 in the PE optional header
    let entry_rva = u32::from_le_bytes([
        data[opt_start + 16], data[opt_start + 17],
        data[opt_start + 18], data[opt_start + 19],
    ]);
    Ok(KERNEL_LOAD_ADDR + entry_rva as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pe_entry_found() {
        let data = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();
        assert!(is_pe_image(&data));
        let entry = parse_pe_entry(&data).unwrap();
        assert_eq!(entry, KERNEL_LOAD_ADDR + KERNEL_PE_ENTRY_OFFSET);
    }
}
