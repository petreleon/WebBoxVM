use super::super::builder::{DtbBuilder, append_two_cell_prop, be_u32_array};
use super::super::*;

pub(super) fn add_interrupt_controller(builder: &mut DtbBuilder) {
    builder.begin_node("intc@8000000");
    builder.prop("compatible", b"arm,gic-v3\0");
    builder.prop("interrupt-controller", &[]);
    builder.prop_u32("#interrupt-cells", 3);
    builder.prop_u32("phandle", 1);

    let mut gic_reg = Vec::new();
    append_two_cell_prop(&mut gic_reg, GICD_BASE, GICD_SIZE);
    append_two_cell_prop(&mut gic_reg, GICR_BASE, GICR_SIZE);
    builder.prop("reg", &gic_reg);
    builder.end_node();
}

pub(super) fn add_timer(builder: &mut DtbBuilder) {
    builder.begin_node("timer");
    builder.prop("compatible", b"arm,armv8-timer\0");
    let timer_irqs: [u32; 12] = [1, 13, 0xf08, 1, 14, 0xf08, 1, 11, 0xf08, 1, 10, 0xf08];
    builder.prop("interrupts", &be_u32_array(&timer_irqs));
    builder.end_node();
}
