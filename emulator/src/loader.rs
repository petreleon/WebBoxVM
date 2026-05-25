//! Load an ARM64 Linux kernel image into emulator RAM.

use crate::bus::SystemBus;
use std::fs;

pub const KERNEL_LOAD: u64 = 0x4008_0000;

/// ARM64 kernel Image header (64 bytes).
struct KernelHeader {
    _code0: u32,
    _code1: u32,
    text_offset: u64,
    image_size: u64,
    _flags: u64,
    _res1: u64,
    _res2: u64,
    _magic: u32,
    _res3: u32,
}

/// Parse header and load kernel at `KERNEL_LOAD`.
/// Returns the physical entry point address.
pub fn load_kernel(bus: &mut SystemBus, path: &str) -> Result<u64, &'static str> {
    let data = fs::read(path).map_err(|_| "read failed")?;
    let header = parse_header(&data)?;

    // If this is a PE/EFI image, find the ARM64 entry point
    let entry = if is_pe(&data) {
        parse_pe_entry(&data)?
    } else {
        KERNEL_LOAD + header.text_offset
    };

    let load_size = if header.image_size > 0 {
        header.image_size as usize
    } else {
        data.len()
    };

    // Copy kernel payload into RAM starting at KERNEL_LOAD
    // For PE files, file offset N maps to KERNEL_LOAD + N
    let payload = &data[..load_size.min(data.len())];
    for (i, &byte) in payload.iter().enumerate() {
        bus.write(KERNEL_LOAD + i as u64, 1, byte as u64);
    }

    Ok(entry)
}

/// Load raw instruction bytes directly into RAM at `KERNEL_LOAD`.
/// Returns `KERNEL_LOAD` as the entry point.
pub fn load_raw_image(bus: &mut SystemBus, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        bus.write(KERNEL_LOAD + i as u64, 1, byte as u64);
    }
}

fn parse_header(data: &[u8]) -> Result<KernelHeader, &'static str> {
    if data.len() < 64 {
        return Err("file too small");
    }
    let read_u32 = |o: usize| u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
    let read_u64 = |o: usize| {
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[o..o + 8]);
        u64::from_le_bytes(b)
    };
    let magic = read_u32(56);
    if magic != 0x644d5241 {
        return Err("bad ARM64 magic");
    }
    Ok(KernelHeader {
        _code0: read_u32(0),
        _code1: read_u32(4),
        text_offset: read_u64(8),
        image_size: read_u64(16),
        _flags: read_u64(24),
        _res1: read_u64(32),
        _res2: read_u64(40),
        _magic: read_u32(56),
        _res3: read_u32(60),
    })
}

fn is_pe(data: &[u8]) -> bool {
    data.len() > 0x40 + 4 && &data[0x40..0x44] == b"PE\0\0"
}

fn parse_pe_entry(data: &[u8]) -> Result<u64, &'static str> {
    let pe_offset = 0x40;
    let opt_start = pe_offset + 24;
    if data.len() < opt_start + 16 {
        return Err("PE truncated");
    }
    let entry_rva = u32::from_le_bytes([data[opt_start + 16], data[opt_start + 17], data[opt_start + 18], data[opt_start + 19]]);
    Ok(KERNEL_LOAD + entry_rva as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pe_entry_found() {
        let data = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();
        assert!(is_pe(&data));
        let entry = parse_pe_entry(&data).unwrap();
        assert_eq!(entry, KERNEL_LOAD + 0x01da7ee0);
    }
}
