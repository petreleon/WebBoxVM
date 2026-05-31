use emulator::dtb::build_dtb;

fn main() {
    let dtb = build_dtb(
        0x4000_0000,
        0x4000_0000,
        None,
        None,
        Some("console=ttyAMA0"),
    );
    std::fs::write("/tmp/webboxvm.dtb", dtb).unwrap();
}
