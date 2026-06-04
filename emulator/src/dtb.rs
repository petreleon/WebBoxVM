//! Device Tree Blob (DTB) generator for Linux boot.
//!
//! The DTB describes the virtual hardware that the bootloader passes to Linux.

use crate::bus::SystemBus;
use crate::constants::*;

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
    let mut builder = DtbBuilder::new();
    nodes::build_tree(
        &mut builder,
        mem_start,
        mem_size,
        initrd_start,
        initrd_end,
        bootargs,
    );
    builder.finish()
}

/// Write a DTB into emulator memory at `addr`.
pub fn load_dtb(bus: &mut SystemBus, addr: u64, dtb: &[u8]) {
    let _ = bus.mem.write_bytes(addr, dtb);
}

#[cfg(test)]
mod tests;
