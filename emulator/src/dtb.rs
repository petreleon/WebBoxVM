//! Device Tree Blob (DTB) generator for Linux boot.
//!
//! The DTB describes the virtual hardware that the bootloader passes to Linux.

use crate::constants::*;
use crate::platform::virt::SystemBus;

mod builder;
mod header;
mod nodes;

use builder::DtbBuilder;

pub fn build_dtb(
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) -> Vec<u8> {
    build_dtb_with_num_cores(mem_start, mem_size, initrd_start, initrd_end, bootargs, 1)
}

pub fn build_dtb_with_num_cores(
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
    num_cores: usize,
) -> Vec<u8> {
    build_dtb_with_boot_media_device_and_num_cores(
        mem_start,
        mem_size,
        initrd_start,
        initrd_end,
        bootargs,
        true,
        num_cores,
    )
}

pub fn build_dtb_with_boot_media_device(
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
    advertise_boot_media: bool,
) -> Vec<u8> {
    build_dtb_with_boot_media_device_and_num_cores(
        mem_start,
        mem_size,
        initrd_start,
        initrd_end,
        bootargs,
        advertise_boot_media,
        1,
    )
}

pub fn build_dtb_with_boot_media_device_and_num_cores(
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
    advertise_boot_media: bool,
    num_cores: usize,
) -> Vec<u8> {
    assert!(num_cores > 0, "num_cores must be at least 1");
    assert!(
        num_cores <= GICR_MAX_CPUS,
        "num_cores exceeds the redistributor MMIO aperture"
    );
    let mut builder = DtbBuilder::new();
    nodes::build_tree(
        &mut builder,
        mem_start,
        mem_size,
        initrd_start,
        initrd_end,
        bootargs,
        advertise_boot_media,
        num_cores,
    );
    builder.finish()
}

/// Write a DTB into emulator memory at `addr`.
pub fn load_dtb(bus: &mut SystemBus, addr: u64, dtb: &[u8]) {
    let _ = bus.mem.write_bytes(addr, dtb);
}

#[cfg(test)]
mod tests;
