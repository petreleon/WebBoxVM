//! Boot test: loads an ARM64 kernel or ISO and checks for UART output.
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/Image
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/debian-arm64-netinst.iso

use emulator::boot::BootContext;
use emulator::loader::iso::load_iso_boot_image;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel_path = env::args().nth(1).unwrap_or_else(|| {
        env::var("WEBBOXVM_KERNEL").unwrap_or_else(|_| ".artifacts/Image".to_string())
    });

    let t0 = Instant::now();
    println!("=== WebBoxVM Boot Test ===");

    let image = fs::read(&kernel_path)?;
    println!(
        "Image loaded: {:.1} MB in {:.1}s",
        image.len() as f64 / 1e6,
        t0.elapsed().as_secs_f32()
    );

    let mut ctx = if is_iso_path(&kernel_path) {
        let boot =
            load_iso_boot_image(&image).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        println!("ISO kernel: {}", boot.kernel_path);
        println!("ISO initrd: {}", boot.initrd_paths.join(", "));
        println!("ISO bootargs: {}", boot.bootargs);
        BootContext::new_with_initrd_and_bootargs(&boot.kernel, 1, &boot.initrd, &boot.bootargs)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    } else {
        BootContext::new(&image, 1).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    };
    println!("BootContext ready in {:.1}s", t0.elapsed().as_secs_f32());

    // ── EFI phase ──
    let efi_steps = ctx.run_efi_phase(5_000_000);
    let u1 = ctx.uart_output();
    println!("EFI phase: {} steps in {:.1}s, PC=0x{:x}, UART={}B",
        efi_steps, t0.elapsed().as_secs_f32(), ctx.pc(), u1.len());
    if !u1.is_empty() {
        println!("  EFI output: {:?}", &u1[..u1.len().min(200)]);
    }

    // ── Kernel phase, checking UART periodically ──
    let mut last_uart = u1.len();
    let chunks = env_usize("BOOT_TEST_CHUNKS", 10);
    let per_chunk = env_usize("BOOT_TEST_STEPS", 2_000_000);

    for i in 0..chunks {
        ctx.run_kernel_phase(per_chunk);
        let uart = ctx.uart_output();
        let new_bytes = uart.len().saturating_sub(last_uart);
        let elapsed = t0.elapsed().as_secs_f32();

        if new_bytes > 0 {
            let start = last_uart.min(uart.len());
            let end = (start + new_bytes).min(uart.len());
            let preview = &uart[start..end];
            println!("Kernel chunk {}: +{}B UART in {:.1}s, PC=0x{:x}, total UART={}B",
                i + 1, new_bytes, elapsed, ctx.pc(), uart.len());
            // Show the new text, sanitizing control chars
            let clean: String = preview.chars()
                .map(|c| if c.is_ascii_graphic() || c == '\n' || c == '\r' || c == ' ' { c } else { '.' })
                .collect();
            println!("  {:?}", clean.chars().take(200).collect::<String>());
        } else {
            println!("Kernel chunk {}: 0B new UART in {:.1}s, PC=0x{:x}",
                i + 1, elapsed, ctx.pc());

            // If PC hasn't moved, something is stuck
            if i > 0 {
                println!("  Warning: no new output for {}M steps", (i+1) * 2);
            }
        }
        last_uart = uart.len();

        // Early exit if we got meaningful output
        if uart.len() > 500 { break; }
    }

    println!("\nTotal: {:.1}s, {} steps, PC=0x{:x}, UART={}B",
        t0.elapsed().as_secs_f32(), ctx.total_steps(), ctx.pc(), ctx.uart_output().len());
    Ok(())
}

fn is_iso_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("iso"))
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
