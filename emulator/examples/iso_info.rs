//! Print the kernel/initrd selected from an ARM64 Linux ISO.

use emulator::loader::iso::load_iso_boot_image;
use std::env;
use std::error::Error;
use std::fs;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let iso_path = env::args()
        .nth(1)
        .unwrap_or_else(|| ".artifacts/debian-arm64-netinst.iso".to_string());
    let image = fs::read(&iso_path)?;
    let boot =
        load_iso_boot_image(&image).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    println!("iso: {iso_path}");
    println!("kernel: {} ({} bytes)", boot.kernel_path, boot.kernel.len());
    println!(
        "initrd: {} ({} bytes)",
        boot.initrd_paths.join(", "),
        boot.initrd.len()
    );
    println!("bootargs: {}", boot.bootargs);

    Ok(())
}
