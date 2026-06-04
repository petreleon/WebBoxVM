use super::super::builder::DtbBuilder;
use super::{clocks_uart, interrupt, memory, virtio_cpu};

pub(super) fn build_tree(
    builder: &mut DtbBuilder,
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) {
    builder.begin_node("");
    builder.prop_u32("#address-cells", 2);
    builder.prop_u32("#size-cells", 2);
    builder.prop("model", b"WebBoxVM\0");
    builder.prop("compatible", b"linux,dummy-virt\0webboxvm,virt\0");
    builder.prop_u32("interrupt-parent", 1);

    memory::add_memory(builder, mem_start, mem_size);
    memory::add_chosen(builder, initrd_start, initrd_end, bootargs);
    interrupt::add_interrupt_controller(builder);
    interrupt::add_timer(builder);
    clocks_uart::add_fixed_clocks(builder);
    clocks_uart::add_uart(builder);
    virtio_cpu::add_virtio_devices(builder);
    virtio_cpu::add_cpus(builder);

    builder.end_node();
    builder.end_tree();
}
