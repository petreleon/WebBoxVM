use super::*;

#[test]
fn test_fdt_header_verification_decoding() {
    use crate::loader::kernel::load_kernel;
    let mut bus = SystemBus::new();
    load_kernel(
        &mut bus,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../.artifacts/Image"),
    )
    .unwrap();

    let addrs = [0x41e29d90u64, 0x41e29d94u64, 0x41e29d98u64];
    for &addr in &addrs {
        let raw = bus.mem.read(addr, 4).unwrap() as u32;
        let instr = decode(raw);
        println!("ADDR={:#x} RAW={:#010x} DECODED={:?}", addr, raw, instr);
    }

    println!("\n--- Relocated loop diagnostics at 0x400b6e50 ---");
    for offset in (0..0x90).step_by(4) {
        let addr = 0x400b6e50u64 + offset;
        let raw = bus.mem.read(addr, 4).unwrap() as u32;
        let instr = decode(raw);
        println!("ADDR={:#x} RAW={:#010x} DECODED={:?}", addr, raw, instr);
    }
}
