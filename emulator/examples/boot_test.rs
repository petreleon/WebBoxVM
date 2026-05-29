//! Boot test: loads the Debian ARM64 kernel and checks for UART output.
//! Run: cargo run --example boot_test --release

use emulator::boot::BootContext;
use std::fs;
use std::time::Instant;

fn main() {
    let kernel_path = "/Users/petreleon/code/WebBoxVM/Image.gz";

    let t0 = Instant::now();
    println!("=== WebBoxVM Boot Test ===");

    let kernel = fs::read(kernel_path).expect("read kernel Image.gz");
    println!("Kernel loaded: {:.1} MB in {:.1}s", kernel.len() as f64 / 1e6, t0.elapsed().as_secs_f32());

    let mut ctx = BootContext::new(&kernel, 1).expect("BootContext::new");
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
    let chunks = 10;
    let per_chunk = 2_000_000;

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
}
