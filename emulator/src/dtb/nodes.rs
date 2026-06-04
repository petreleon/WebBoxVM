mod clocks_uart;
mod interrupt;
mod memory;
mod root;
mod virtio_cpu;

use super::builder::DtbBuilder;

pub(super) fn build_tree(
    builder: &mut DtbBuilder,
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) {
    root::build_tree(
        builder,
        mem_start,
        mem_size,
        initrd_start,
        initrd_end,
        bootargs,
    );
}
