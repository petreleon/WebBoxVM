use super::super::builder::{DtbBuilder, append_two_cell_prop, c_string};

pub(super) fn add_memory(builder: &mut DtbBuilder, mem_start: u64, mem_size: u64) {
    builder.begin_node("memory@40000000");
    builder.prop("device_type", b"memory\0");
    let mut reg = Vec::new();
    append_two_cell_prop(&mut reg, mem_start, mem_size);
    builder.prop("reg", &reg);
    builder.end_node();
}

pub(super) fn add_chosen(
    builder: &mut DtbBuilder,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) {
    builder.begin_node("chosen");
    builder.prop("stdout-path", b"/uart@9000000:115200n8\0");
    if let Some(args) = bootargs {
        builder.prop("bootargs", &c_string(args));
    }
    if let (Some(start), Some(end)) = (initrd_start, initrd_end) {
        builder.prop("linux,initrd-start", &start.to_be_bytes());
        builder.prop("linux,initrd-end", &end.to_be_bytes());
    }
    builder.end_node();
}
