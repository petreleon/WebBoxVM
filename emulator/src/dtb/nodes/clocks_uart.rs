use super::super::builder::{DtbBuilder, append_two_cell_prop, be_u32_array};
use super::super::*;

pub(super) fn add_fixed_clocks(builder: &mut DtbBuilder) {
    add_clock(builder, "uartclk", 2);
    add_clock(builder, "apb-pclk", 3);
}

pub(super) fn add_uart(builder: &mut DtbBuilder) {
    builder.begin_node("uart@9000000");
    builder.prop("compatible", b"arm,pl011\0arm,primecell\0");
    let mut reg = Vec::new();
    append_two_cell_prop(&mut reg, UART_BASE, UART_SIZE);
    builder.prop("reg", &reg);
    builder.prop_u32("clock-frequency", 24_000_000);
    builder.prop("clocks", &be_u32_array(&[2, 3]));
    builder.prop("clock-names", b"uartclk\0apb_pclk\0");
    builder.prop_u32("current-speed", 115_200);
    builder.prop("interrupts", &be_u32_array(&[0, 1, 4]));
    builder.end_node();
}

fn add_clock(builder: &mut DtbBuilder, name: &str, phandle: u32) {
    builder.begin_node(name);
    builder.prop("compatible", b"fixed-clock\0");
    builder.prop_u32("#clock-cells", 0);
    builder.prop_u32("clock-frequency", 24_000_000);
    builder.prop_u32("phandle", phandle);
    builder.end_node();
}
