//! cpio `newc` initrd parser, builder, and memory loader.
//!
//! The Linux initrd is a cpio archive loaded into RAM. The `newc` format uses
//! fixed-width ASCII hexadecimal fields, which keeps the builder simple and
//! makes generated archives easy to inspect.

use crate::platform::virt::SystemBus;

mod builder;
mod node;
mod parser;

pub use builder::{build_cpio, build_cpio_nodes};
pub use node::{CpioEntry, CpioNode};
pub use parser::parse_cpio;

/// Load a cpio archive into emulator memory at `addr`.
pub fn load_initrd(bus: &mut SystemBus, addr: u64, data: &[u8]) {
    let _ = bus.mem.write_bytes(addr, data);
}

pub(crate) fn pad_to_4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

pub(crate) fn round_up_to_4(n: usize) -> usize {
    (n + 3) & !3
}

#[cfg(test)]
mod tests;
