use super::super::builder::DtbBuilder;
use super::{clocks_uart, interrupt, memory, virtio_cpu};

pub(super) fn build_tree(
    builder: &mut DtbBuilder,
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
    advertise_boot_media: bool,
    num_cores: usize,
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
    virtio_cpu::add_virtio_devices(builder, advertise_boot_media);
    add_psci(builder);
    virtio_cpu::add_cpus(builder, num_cores);

    builder.end_node();
    builder.end_tree();
}

fn add_psci(builder: &mut DtbBuilder) {
    builder.begin_node("psci");
    builder.prop("compatible", b"arm,psci-0.2\0");
    builder.prop("method", b"hvc\0");
    builder.end_node();
}
