mod debug;
mod kernel;
mod prompt_script;
mod util;

use emulator::boot::BootContext;
use emulator::loader::iso::load_iso_boot_image;
use std::env;
use std::fs;
use std::io;
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
    println!("BootContext ready in {:.1}s", t0.elapsed().as_secs_f32());

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
) -> Result<(BootContext, Option<Vec<u8>>), Box<dyn std::error::Error>> {
    if util::is_iso_path(kernel_path) {
        build_iso_context(image)
    } else {
        let ctx =
            BootContext::new(image, 1).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok((ctx, None))
    }
}

fn build_iso_context(
    image: &[u8],
) -> Result<(BootContext, Option<Vec<u8>>), Box<dyn std::error::Error>> {
    let boot =
        load_iso_boot_image(image).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    println!("ISO kernel: {}", boot.kernel_path);
    println!("ISO initrd: {}", boot.initrd_paths.join(", "));

    let bootargs = match env::var("BOOT_TEST_EXTRA_BOOTARGS") {
        Ok(extra) if !extra.trim().is_empty() => format!("{} {}", boot.bootargs, extra.trim()),
        _ => boot.bootargs.clone(),
    };
    println!("ISO bootargs: {}", bootargs);

    let expected_initrd = boot.initrd.clone();
    let mut ctx =
        BootContext::new_with_initrd_and_bootargs(&boot.kernel, 1, &boot.initrd, &bootargs)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    ctx.attach_virtio_block(image);
    Ok((ctx, Some(expected_initrd)))
}

fn run_efi_phase(ctx: &mut BootContext, t0: &Instant) {
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
