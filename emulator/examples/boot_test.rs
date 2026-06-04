//! Boot test: loads an ARM64 kernel or ISO and checks for UART output.
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/Image
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/debian-arm64-netinst.iso

mod boot_test_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    boot_test_app::run()
}
