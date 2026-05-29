//! Device Tree Blob (DTB) generator for Linux boot.
//!
//! The DTB is a binary data structure that describes the virtual hardware
//! to the Linux kernel.  It replaces the old "board files" — instead of
//! hardcoding platform details in the kernel, the bootloader passes a DTB.
//!
//! ## DTB layout
//!
//! ```text
//!  Header (40 bytes)
//!  Memory reservation block (ends with 16 zero bytes)
//!  Structure block (nodes, properties — FDT_BEGIN_NODE, FDT_PROP, etc.)
//!  Strings block (pool of null-terminated property name strings)
//! ```
//!
//! Reference: https://www.devicetree.org/specifications/

use crate::bus::SystemBus;
use crate::constants::*;

/// Build a minimal DTB describing the emulated machine.
///
/// Nodes created:
///   - `/` (root) with address/size cells
///   - `memory@40000000` — RAM region
///   - `chosen` — bootargs, initrd location, stdout path
///   - `intc@8000000` — GICv3 interrupt controller
///   - `timer` — ARMv8 architected timer
///   - `uart@9000000` — PL011 serial console
///   - `cpus/cpu@0` — single Cortex-A72 core
pub fn build_dtb(
    mem_start: u64,
    mem_size: u64,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) -> Vec<u8> {
    let mut strings = Vec::new();
    let mut struct_block = Vec::new();

    // ── DTB block writers ──

    let push_token = |block: &mut Vec<u8>, token: u32| {
        block.extend_from_slice(&token.to_be_bytes());
    };

    let push_name = |block: &mut Vec<u8>, name: &str| {
        block.extend_from_slice(name.as_bytes());
        block.push(0); // null terminator
        pad_to_4(block);
    };

    let push_prop = |block: &mut Vec<u8>, strings: &mut Vec<u8>, name: &str, value: &[u8]| {
        let nameoff = strings.len() as u32;
        strings.extend_from_slice(name.as_bytes());
        strings.push(0);
        push_token(block, FDT_PROP);
        block.extend_from_slice(&(value.len() as u32).to_be_bytes());
        block.extend_from_slice(&nameoff.to_be_bytes());
        block.extend_from_slice(value);
        pad_to_4(block);
    };

    // ── Root node (/) ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, ""); // root = empty name

    push_prop(&mut struct_block, &mut strings, "#address-cells", &2u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "#size-cells", &2u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "model", b"WebBoxVM\0");
    push_prop(&mut struct_block, &mut strings, "compatible", b"webboxvm,virt\0");
    push_prop(&mut struct_block, &mut strings, "interrupt-parent", &1u32.to_be_bytes()); // phandle → intc

    // ── memory@40000000 ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "memory@40000000");
    push_prop(&mut struct_block, &mut strings, "device_type", b"memory\0");
    // reg = <0x00000000 mem_start  0x00000000 mem_size> (two-cell address/size)
    let mut reg = Vec::new();
    reg.extend_from_slice(&0u32.to_be_bytes());
    reg.extend_from_slice(&(mem_start as u32).to_be_bytes());
    reg.extend_from_slice(&0u32.to_be_bytes());
    reg.extend_from_slice(&(mem_size as u32).to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "reg", &reg);
    push_token(&mut struct_block, FDT_END_NODE);

    // ── chosen ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "chosen");
    push_prop(&mut struct_block, &mut strings, "stdout-path", b"/uart@9000000\0");
    if let Some(args) = bootargs {
        push_prop(&mut struct_block, &mut strings, "bootargs", args.as_bytes());
    }
    if let (Some(start), Some(end)) = (initrd_start, initrd_end) {
        push_prop(&mut struct_block, &mut strings, "linux,initrd-start", &start.to_be_bytes());
        push_prop(&mut struct_block, &mut strings, "linux,initrd-end", &end.to_be_bytes());
    }
    push_token(&mut struct_block, FDT_END_NODE);

    // ── intc@8000000 (GICv3) ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "intc@8000000");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,gic-v3\0");
    push_prop(&mut struct_block, &mut strings, "interrupt-controller", &[]);
    push_prop(&mut struct_block, &mut strings, "#interrupt-cells", &3u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "phandle", &1u32.to_be_bytes()); // phandle=1
    // reg: GICD at 0x08000000 size 0x10000; GICR at 0x080A0000 size 0xF60000
    let mut gic_reg = Vec::new();
    append_two_cell_prop(&mut gic_reg, GICD_BASE, GICD_SIZE);
    append_two_cell_prop(&mut gic_reg, GICR_BASE, GICR_SIZE);
    push_prop(&mut struct_block, &mut strings, "reg", &gic_reg);
    push_token(&mut struct_block, FDT_END_NODE);

    // ── timer ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "timer");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,armv8-timer\0");
    // 4 PPIs: secure=13, non-secure=14, virt=11, hyp=10  (flags=0xF08 each)
    let timer_irqs: [u32; 12] = [
        1, 13, 0xf08,   // Secure Physical PPI
        1, 14, 0xf08,   // Non-Secure Physical PPI
        1, 11, 0xf08,   // Virtual PPI
        1, 10, 0xf08,   // Hypervisor Physical PPI
    ];
    let mut timer_irq_bytes = Vec::new();
    for val in &timer_irqs {
        timer_irq_bytes.extend_from_slice(&val.to_be_bytes());
    }
    push_prop(&mut struct_block, &mut strings, "interrupts", &timer_irq_bytes);
    push_token(&mut struct_block, FDT_END_NODE);

    // ── uart@9000000 ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "uart@9000000");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,pl011\0arm,primecell\0");
    let mut uart_reg = Vec::new();
    append_two_cell_prop(&mut uart_reg, UART_BASE, UART_SIZE);
    push_prop(&mut struct_block, &mut strings, "reg", &uart_reg);
    push_prop(&mut struct_block, &mut strings, "clock-frequency", &24_000_000u32.to_be_bytes());
    // UART interrupt: SPI #1, edge-triggered (flags=4)
    let uart_irqs: [u32; 3] = [0, 1, 4];
    let mut uart_bytes = Vec::new();
    for val in &uart_irqs {
        uart_bytes.extend_from_slice(&val.to_be_bytes());
    }
    push_prop(&mut struct_block, &mut strings, "interrupts", &uart_bytes);
    push_token(&mut struct_block, FDT_END_NODE);

    // ── cpus ──
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "cpus");
    push_prop(&mut struct_block, &mut strings, "#address-cells", &1u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "#size-cells", &0u32.to_be_bytes());

    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_name(&mut struct_block, "cpu@0");
    push_prop(&mut struct_block, &mut strings, "device_type", b"cpu\0");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,armv8\0");
    push_prop(&mut struct_block, &mut strings, "reg", &0u32.to_be_bytes()); // CPU ID = 0
    push_token(&mut struct_block, FDT_END_NODE);

    push_token(&mut struct_block, FDT_END_NODE); // end cpus

    // ── Close root node; end structure ──
    push_token(&mut struct_block, FDT_END_NODE);
    push_token(&mut struct_block, FDT_END);

    // ── Assemble the DTB binary ──
    pad_to_4(&mut strings);
    pad_to_4(&mut struct_block);

    let header_size: u32 = 40;
    let mem_rsvmap_size: u32 = 16; // end marker = two zero u64s
    let off_mem_rsvmap = header_size;
    let off_dt_struct = off_mem_rsvmap + mem_rsvmap_size;
    let off_dt_strings = off_dt_struct + struct_block.len() as u32;
    let totalsize = off_dt_strings + strings.len() as u32;

    let mut dtb = Vec::new();
    dtb.extend_from_slice(&FDT_MAGIC.to_be_bytes());
    dtb.extend_from_slice(&totalsize.to_be_bytes());
    dtb.extend_from_slice(&off_dt_struct.to_be_bytes());
    dtb.extend_from_slice(&off_dt_strings.to_be_bytes());
    dtb.extend_from_slice(&off_mem_rsvmap.to_be_bytes());
    dtb.extend_from_slice(&FDT_VERSION.to_be_bytes());
    dtb.extend_from_slice(&FDT_LAST_COMP_VERSION.to_be_bytes());
    dtb.extend_from_slice(&0u32.to_be_bytes()); // boot_cpuid_phys = 0
    dtb.extend_from_slice(&(strings.len() as u32).to_be_bytes()); // size_dt_strings
    dtb.extend_from_slice(&(struct_block.len() as u32).to_be_bytes()); // size_dt_struct

    // Memory reservation block (empty)
    dtb.extend_from_slice(&0u64.to_be_bytes());
    dtb.extend_from_slice(&0u64.to_be_bytes());

    // Structure block
    dtb.extend_from_slice(&struct_block);

    // Strings block
    dtb.extend_from_slice(&strings);

    dtb
}

/// Write a DTB into emulator memory at `addr`.
pub fn load_dtb(bus: &mut SystemBus, addr: u64, dtb: &[u8]) {
    for (i, &byte) in dtb.iter().enumerate() {
        bus.write(addr + i as u64, 1, byte as u64);
    }
}

// ── Helpers ──

/// Append a two-cell address/size pair (8 bytes each, total 16 per entry).
fn append_two_cell_prop(bytes: &mut Vec<u8>, addr: u64, size: u64) {
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&(addr as u32).to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&(size as u32).to_be_bytes());
}

fn pad_to_4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

#[cfg(test)]
mod tests;
