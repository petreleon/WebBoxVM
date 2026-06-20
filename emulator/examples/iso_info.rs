//! Print the kernel/initrd selected from an ARM64 Linux ISO.

use emulator::host::native::read_iso_boot_info;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let iso_path = env::args()
        .nth(1)
        .unwrap_or_else(|| ".artifacts/debian-arm64-netinst.iso".to_string());
    let (info, kernel_len, initrd_len) = read_iso_boot_info(&iso_path)?;

    println!("iso: {iso_path}");
    println!("kernel: {} ({} bytes)", info.kernel_path, kernel_len);
    println!(
        "initrd: {} ({} bytes)",
        info.initrd_paths.join(", "),
        initrd_len
    );
    println!("bootargs: {}", info.bootargs);

    Ok(())
}
