use super::super::builder::{DtbBuilder, append_two_cell_prop, be_u32_array};
use super::super::*;

pub(super) fn add_virtio_devices(builder: &mut DtbBuilder, advertise_boot_media: bool) {
    if advertise_boot_media {
        add_virtio_mmio(builder, "virtio_blk@a000000", VIRTIO_BLK_BASE, 16);
    }
    add_virtio_mmio(builder, "virtio_blk@a001000", VIRTIO_DISK_BASE, 17);
    add_virtio_mmio(builder, "virtio_net@a002000", VIRTIO_NET_BASE, 18);
}

pub(super) fn add_cpus(builder: &mut DtbBuilder) {
    builder.begin_node("cpus");
    builder.prop_u32("#address-cells", 1);
    builder.prop_u32("#size-cells", 0);

    builder.begin_node("cpu@0");
    builder.prop("device_type", b"cpu\0");
    builder.prop("compatible", b"arm,armv8\0");
    builder.prop("reg", &0u32.to_be_bytes());
    builder.end_node();

    builder.end_node();
}

fn add_virtio_mmio(builder: &mut DtbBuilder, name: &str, base: u64, spi: u32) {
    builder.begin_node(name);
    builder.prop("compatible", b"virtio,mmio\0");
    let mut reg = Vec::new();
    append_two_cell_prop(&mut reg, base, VIRTIO_BLK_SIZE);
    builder.prop("reg", &reg);
    builder.prop("interrupts", &be_u32_array(&[0, spi, 4]));
    builder.prop("dma-coherent", &[]);
    builder.end_node();
}
