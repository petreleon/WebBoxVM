//! Minimal Device Tree Blob (DTB) generator for Linux boot.

use crate::bus::SystemBus;

const FDT_MAGIC: u32 = 0xd00dfeed;
const FDT_VERSION: u32 = 17;
const FDT_LAST_COMP_VERSION: u32 = 16;

const FDT_BEGIN_NODE: u32 = 0x0000_0001;
const FDT_END_NODE: u32 = 0x0000_0002;
const FDT_PROP: u32 = 0x0000_0003;
const FDT_END: u32 = 0x0000_0009;

/// Build a minimal DTB with memory region and optional initrd.
pub fn build_dtb(mem_start: u64, mem_size: u64, initrd_start: Option<u64>, initrd_end: Option<u64>, bootargs: Option<&str>) -> Vec<u8> {
    let mut strings = Vec::new();
    let mut struct_block = Vec::new();

    let push_token = |block: &mut Vec<u8>, token: u32| {
        block.extend_from_slice(&token.to_be_bytes());
    };

    let push_str = |block: &mut Vec<u8>, s: &str| {
        block.extend_from_slice(s.as_bytes());
        block.push(0);
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

    // / node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "");

    // #address-cells = <2>
    push_prop(&mut struct_block, &mut strings, "#address-cells", &2u32.to_be_bytes());
    // #size-cells = <2>
    push_prop(&mut struct_block, &mut strings, "#size-cells", &2u32.to_be_bytes());
    // model
    push_prop(&mut struct_block, &mut strings, "model", b"WebBoxVM\0");
    // compatible
    push_prop(&mut struct_block, &mut strings, "compatible", b"webboxvm,virt\0");
    // interrupt-parent
    push_prop(&mut struct_block, &mut strings, "interrupt-parent", &1u32.to_be_bytes());

    // memory node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "memory@40000000");
    push_prop(&mut struct_block, &mut strings, "device_type", b"memory\0");
    let mut reg = Vec::new();
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(mem_start as u32).to_be_bytes());
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(mem_size as u32).to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "reg", &reg);
    push_token(&mut struct_block, FDT_END_NODE);

    // chosen node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "chosen");
    push_prop(&mut struct_block, &mut strings, "stdout-path", b"/uart@9000000\0");
    if let Some(args) = bootargs {
        push_prop(&mut struct_block, &mut strings, "bootargs", args.as_bytes());
    }
    if let (Some(start), Some(end)) = (initrd_start, initrd_end) {
        push_prop(&mut struct_block, &mut strings, "linux,initrd-start", &start.to_be_bytes());
        push_prop(&mut struct_block, &mut strings, "linux,initrd-end", &end.to_be_bytes());
    }
    push_token(&mut struct_block, FDT_END_NODE);

    // intc@8000000 (GICv3)
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "intc@8000000");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,gic-v3\0");
    push_prop(&mut struct_block, &mut strings, "interrupt-controller", &[]);
    push_prop(&mut struct_block, &mut strings, "#interrupt-cells", &3u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "phandle", &1u32.to_be_bytes());
    let mut reg = Vec::new();
    // GICD: 0x08000000, size 0x10000
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0x0800_0000u32).to_be_bytes());
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0x10000u32).to_be_bytes());
    // GICR: 0x080a0000, size 0xf60000
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0x080A_0000u32).to_be_bytes());
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0xf60000u32).to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "reg", &reg);
    push_token(&mut struct_block, FDT_END_NODE);

    // timer node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "timer");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,armv8-timer\0");
    let mut timer_irqs = Vec::new();
    // Secure Phys PPI 13
    timer_irqs.extend_from_slice(&1u32.to_be_bytes());
    timer_irqs.extend_from_slice(&13u32.to_be_bytes());
    timer_irqs.extend_from_slice(&0xf08u32.to_be_bytes());
    // Non-Secure Phys PPI 14
    timer_irqs.extend_from_slice(&1u32.to_be_bytes());
    timer_irqs.extend_from_slice(&14u32.to_be_bytes());
    timer_irqs.extend_from_slice(&0xf08u32.to_be_bytes());
    // Virt PPI 11
    timer_irqs.extend_from_slice(&1u32.to_be_bytes());
    timer_irqs.extend_from_slice(&11u32.to_be_bytes());
    timer_irqs.extend_from_slice(&0xf08u32.to_be_bytes());
    // Hyp PPI 10
    timer_irqs.extend_from_slice(&1u32.to_be_bytes());
    timer_irqs.extend_from_slice(&10u32.to_be_bytes());
    timer_irqs.extend_from_slice(&0xf08u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "interrupts", &timer_irqs);
    push_token(&mut struct_block, FDT_END_NODE);

    // uart node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "uart@9000000");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,pl011\0arm,primecell\0");
    let mut reg = Vec::new();
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0x0900_0000u32).to_be_bytes());
    reg.extend_from_slice(&(0u32).to_be_bytes());
    reg.extend_from_slice(&(0x1000u32).to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "reg", &reg);
    push_prop(&mut struct_block, &mut strings, "clock-frequency", &24000000u32.to_be_bytes());
    let mut uart_irqs = Vec::new();
    uart_irqs.extend_from_slice(&0u32.to_be_bytes()); // SPI
    uart_irqs.extend_from_slice(&1u32.to_be_bytes()); // Interrupt 1
    uart_irqs.extend_from_slice(&4u32.to_be_bytes()); // High-level trigger
    push_prop(&mut struct_block, &mut strings, "interrupts", &uart_irqs);
    push_token(&mut struct_block, FDT_END_NODE);

    // cpus node
    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "cpus");
    push_prop(&mut struct_block, &mut strings, "#address-cells", &1u32.to_be_bytes());
    push_prop(&mut struct_block, &mut strings, "#size-cells", &0u32.to_be_bytes());

    push_token(&mut struct_block, FDT_BEGIN_NODE);
    push_str(&mut struct_block, "cpu@0");
    push_prop(&mut struct_block, &mut strings, "device_type", b"cpu\0");
    push_prop(&mut struct_block, &mut strings, "compatible", b"arm,armv8\0");
    push_prop(&mut struct_block, &mut strings, "reg", &0u32.to_be_bytes());
    push_token(&mut struct_block, FDT_END_NODE);

    push_token(&mut struct_block, FDT_END_NODE);

    // End root
    push_token(&mut struct_block, FDT_END_NODE);
    push_token(&mut struct_block, FDT_END);

    // Pad strings to 4-byte
    pad_to_4(&mut strings);
    pad_to_4(&mut struct_block);

    let mem_rsvmap_size = 16; // just the end marker (16 zero bytes: two u64 zeros)
    let header_size = 40u32;
    let off_mem_rsvmap = header_size;
    let off_dt_struct = off_mem_rsvmap + mem_rsvmap_size;
    let off_dt_strings = off_dt_struct + struct_block.len() as u32;
    let totalsize = off_dt_strings + strings.len() as u32;

    println!("BUILD_DTB DIAG: off_dt_struct={} struct_block.len()={} off_dt_strings={} strings.len()={} totalsize={}",
        off_dt_struct, struct_block.len(), off_dt_strings, strings.len(), totalsize);

    let mut dtb = Vec::new();
    dtb.extend_from_slice(&FDT_MAGIC.to_be_bytes());
    dtb.extend_from_slice(&totalsize.to_be_bytes());
    dtb.extend_from_slice(&off_dt_struct.to_be_bytes());
    dtb.extend_from_slice(&off_dt_strings.to_be_bytes());
    dtb.extend_from_slice(&off_mem_rsvmap.to_be_bytes());
    dtb.extend_from_slice(&FDT_VERSION.to_be_bytes());
    dtb.extend_from_slice(&FDT_LAST_COMP_VERSION.to_be_bytes());
    dtb.extend_from_slice(&0u32.to_be_bytes()); // boot_cpuid_phys
    dtb.extend_from_slice(&(strings.len() as u32).to_be_bytes()); // size_dt_strings
    dtb.extend_from_slice(&(struct_block.len() as u32).to_be_bytes()); // size_dt_struct

    // Memory reservation block (empty + end marker: 16 bytes of zeros)
    dtb.extend_from_slice(&0u64.to_be_bytes());
    dtb.extend_from_slice(&0u64.to_be_bytes());

    // Structure block
    dtb.extend_from_slice(&struct_block);

    // Strings block
    dtb.extend_from_slice(&strings);

    dtb
}

/// Write a DTB into memory at `addr`.
pub fn load_dtb(bus: &mut SystemBus, addr: u64, dtb: &[u8]) {
    for (i, &byte) in dtb.iter().enumerate() {
        bus.write(addr + i as u64, 1, byte as u64);
    }
}

fn pad_to_4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

#[cfg(test)]
mod tests;
