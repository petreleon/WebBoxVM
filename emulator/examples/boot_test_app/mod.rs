mod debug;
mod kernel;
mod prompt_script;
mod util;

use emulator::host::native::{NativeBootSource, NativeVm, boot_from_image};
use std::env;
use std::fs;
use std::time::Instant;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let kernel_path = util::kernel_path();
    let t0 = Instant::now();
    println!("=== WebBoxVM Boot Test ===");

    let image = fs::read(&kernel_path)?;
    println!(
        "Image loaded: {:.1} MB in {:.1}s",
        image.len() as f64 / 1e6,
        t0.elapsed().as_secs_f32()
    );

    let (mut ctx, expected_initrd) = build_context(&kernel_path, &image)?;
    println!("Native VM ready in {:.1}s", t0.elapsed().as_secs_f32());

    run_efi_phase(&mut ctx, &t0);
    kernel::run_kernel_chunks(&mut ctx, expected_initrd.as_deref(), &t0);

    println!(
        "\nTotal: {:.1}s, {} steps, PC=0x{:x}, UART={}B",
        t0.elapsed().as_secs_f32(),
        ctx.total_steps(),
        ctx.pc(),
        ctx.uart_output().len()
    );
    debug::print_debug_state(&ctx);
    Ok(())
}

fn build_context(
    kernel_path: &str,
    image: &[u8],
) -> Result<(NativeVm, Option<Vec<u8>>), Box<dyn std::error::Error>> {
    let extra = env::var("BOOT_TEST_EXTRA_BOOTARGS").ok();
    let cores = util::env_usize("BOOT_TEST_CORES", 1).max(1);
    println!("Virtual CPUs: {cores}");
    let boot = boot_from_image(kernel_path, image, cores, extra.as_deref())?;

    if let NativeBootSource::Iso(info) = &boot.source {
        println!("ISO kernel: {}", info.kernel_path);
        println!("ISO initrd: {}", info.initrd_paths.join(", "));
        println!("ISO bootargs: {}", info.bootargs);
    }

    Ok((boot.context, boot.expected_initrd))
}

fn run_efi_phase(ctx: &mut NativeVm, t0: &Instant) {
    let efi_steps = ctx.run_efi_phase(5_000_000);
    let uart = ctx.uart_output();
    println!(
        "EFI phase: {} steps in {:.1}s, PC=0x{:x}, UART={}B",
        efi_steps,
        t0.elapsed().as_secs_f32(),
        ctx.pc(),
        uart.len()
    );
    if !uart.is_empty() {
        println!("  EFI output: {:?}", &uart[..uart.len().min(200)]);
    }
}
