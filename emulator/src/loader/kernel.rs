//! Live kernel loading adapter.
//!
//! Pure kernel image parsing lives in `images::kernel`; this module owns the
//! legacy side effects of copying bytes into RAM and installing EFI tables.

use crate::constants::*;
use crate::images::kernel::parse_kernel_image;
use crate::platform::virt::SystemBus;
use std::fs;

pub const KERNEL_LOAD: u64 = KERNEL_LOAD_ADDR;

pub fn load_kernel(bus: &mut SystemBus, path: &str) -> Result<u64, &'static str> {
    let data = fs::read(path).map_err(|_| "read failed")?;
    let image = parse_kernel_image(&data)?;

    bus.mem
        .write_bytes(KERNEL_LOAD_ADDR, &image.payload)
        .ok_or("kernel image does not fit in guest RAM")?;

    if image.needs_efi_tables {
        crate::efi::setup_efi_tables(bus, KERNEL_LOAD_ADDR, image.image_size, DTB_BASE);
    }

    Ok(image.entry)
}

/// Copy a raw kernel image into RAM.
pub fn load_raw_image(bus: &mut SystemBus, data: &[u8]) {
    let _ = bus.mem.write_bytes(KERNEL_LOAD_ADDR, data);
}
