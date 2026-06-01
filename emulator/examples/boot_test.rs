//! Boot test: loads an ARM64 kernel or ISO and checks for UART output.
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/Image
//! Run: cargo run -p emulator --example boot_test --release -- .artifacts/debian-arm64-netinst.iso

use emulator::boot::BootContext;
use emulator::constants::INITRD_BASE;
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

    let (mut ctx, expected_initrd) = if is_iso_path(&kernel_path) {
        let boot =
            load_iso_boot_image(&image).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        println!("ISO kernel: {}", boot.kernel_path);
        println!("ISO initrd: {}", boot.initrd_paths.join(", "));
        let bootargs = match env::var("BOOT_TEST_EXTRA_BOOTARGS") {
            Ok(extra) if !extra.trim().is_empty() => {
                format!("{} {}", boot.bootargs, extra.trim())
            }
            _ => boot.bootargs.clone(),
        };
        println!("ISO bootargs: {}", bootargs);
        let expected_initrd = boot.initrd.clone();
        let mut ctx =
            BootContext::new_with_initrd_and_bootargs(&boot.kernel, 1, &boot.initrd, &bootargs)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        ctx.attach_virtio_block(&image);
        (ctx, Some(expected_initrd))
    } else {
        (
            BootContext::new(&image, 1).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?,
            None,
        )
    };
    println!("BootContext ready in {:.1}s", t0.elapsed().as_secs_f32());

    // ── EFI phase ──
    let efi_steps = ctx.run_efi_phase(5_000_000);
    let u1 = ctx.uart_output();
    println!(
        "EFI phase: {} steps in {:.1}s, PC=0x{:x}, UART={}B",
        efi_steps,
        t0.elapsed().as_secs_f32(),
        ctx.pc(),
        u1.len()
    );
    if !u1.is_empty() {
        println!("  EFI output: {:?}", &u1[..u1.len().min(200)]);
    }

    // ── Kernel phase, checking UART periodically ──
    let mut last_uart = u1.len();
    let chunks = env_usize("BOOT_TEST_CHUNKS", 10);
    let per_chunk = env_usize("BOOT_TEST_STEPS", 2_000_000);
    let command_script = env::var("BOOT_TEST_COMMANDS")
        .ok()
        .filter(|script| !script.is_empty())
        .map(|script| normalize_boot_test_commands(&script));
    let stop_uart_default = if command_script.is_some() { 0 } else { 500 };
    let stop_uart = env_usize("BOOT_TEST_STOP_UART", stop_uart_default);
    let stop_text = env::var("BOOT_TEST_STOP_TEXT")
        .ok()
        .filter(|text| !text.is_empty());
    let mut commands_sent = false;

    for i in 0..chunks {
        ctx.run_kernel_phase(per_chunk);
        if let Some(expected) = expected_initrd.as_deref() {
            if env::var_os("BOOT_TEST_CHECK_INITRD").is_some() {
                match first_initrd_mismatch(&ctx, expected) {
                    Some(pos) => println!(
                        "  Initrd changed at +0x{pos:x}: expected=0x{:02x} guest=0x{:02x}",
                        expected[pos],
                        ctx.machine
                            .bus
                            .mem
                            .read(INITRD_BASE + pos as u64, 1)
                            .unwrap_or(0)
                    ),
                    None => println!("  Initrd source bytes still match"),
                }
            }
        }
        let uart = ctx.uart_output();
        let new_bytes = uart.len().saturating_sub(last_uart);
        let elapsed = t0.elapsed().as_secs_f32();

        if new_bytes > 0 {
            let start = last_uart.min(uart.len());
            let end = (start + new_bytes).min(uart.len());
            let preview = &uart[start..end];
            println!(
                "Kernel chunk {}: +{}B UART in {:.1}s, PC=0x{:x}, total UART={}B",
                i + 1,
                new_bytes,
                elapsed,
                ctx.pc(),
                uart.len()
            );
            // Show the new text, sanitizing control chars
            let clean: String = preview
                .chars()
                .map(|c| {
                    if c.is_ascii_graphic() || c == '\n' || c == '\r' || c == ' ' {
                        c
                    } else {
                        '.'
                    }
                })
                .collect();
            if env::var_os("BOOT_TEST_UART_FULL").is_some() {
                println!("{clean}");
            } else {
                println!("  {:?}", clean.chars().take(200).collect::<String>());
            }
        } else {
            println!(
                "Kernel chunk {}: 0B new UART in {:.1}s, PC=0x{:x}",
                i + 1,
                elapsed,
                ctx.pc()
            );

            // If PC hasn't moved, something is stuck
            if i > 0 {
                println!("  Warning: no new output for {}M steps", (i + 1) * 2);
            }
        }
        last_uart = uart.len();

        if !commands_sent
            && let Some(script) = command_script.as_deref()
            && uart.contains("webboxvm# ")
        {
            ctx.feed_uart_input(script);
            commands_sent = true;
            println!(
                "Fed {} bytes of UART input from BOOT_TEST_COMMANDS",
                script.len()
            );
        }

        if let Some(text) = stop_text.as_deref()
            && uart.contains(text)
        {
            println!("Stop text matched: {text:?}");
            break;
        }

        // Early exit if we got meaningful output
        if stop_uart > 0 && uart.len() > stop_uart {
            break;
        }
    }

    println!(
        "\nTotal: {:.1}s, {} steps, PC=0x{:x}, UART={}B",
        t0.elapsed().as_secs_f32(),
        ctx.total_steps(),
        ctx.pc(),
        ctx.uart_output().len()
    );
    if env::var_os("BOOT_TEST_DEBUG_STATE").is_some() {
        let cpu = ctx.machine.core(0);
        println!(
            "CPU: cycle={} pstate=0x{:x} irq_pending={} last_irq={} cntp_ctl=0x{:x} cntp_cval={} cntv_ctl=0x{:x} cntv_cval={} elr=0x{:x} esr=0x{:x} far=0x{:x}",
            cpu.sys.cycle_count,
            cpu.pstate.to_u64(),
            cpu.sys.irq_pending,
            cpu.sys.last_irq_id,
            cpu.sys.cntp_ctl_el0,
            cpu.sys.cntp_cval_el0,
            cpu.sys.cntv_ctl_el0,
            cpu.sys.cntv_cval_el0,
            cpu.sys.elr_el1,
            cpu.sys.esr_el1,
            cpu.sys.far_el1,
        );
        println!(
            "GIC: enable0=0x{:08x} enable1=0x{:08x} pending0=0x{:08x} pending1=0x{:08x} group0=0x{:08x} group1=0x{:08x}",
            ctx.machine.bus.gic.enable[0],
            ctx.machine.bus.gic.enable[1],
            ctx.machine.bus.gic.pending[0],
            ctx.machine.bus.gic.pending[1],
            ctx.machine.bus.gic.group[0],
            ctx.machine.bus.gic.group[1],
        );
    }
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

fn normalize_boot_test_commands(script: &str) -> String {
    let mut normalized = script.replace("\\r", "\r").replace("\\n", "\n");
    normalized = normalized.replace('\n', "\r");
    if !normalized.ends_with('\r') {
        normalized.push('\r');
    }
    normalized
}

fn first_initrd_mismatch(ctx: &BootContext, expected: &[u8]) -> Option<usize> {
    let mut guest = vec![0; expected.len()];
    ctx.machine.bus.mem.read_bytes(INITRD_BASE, &mut guest)?;
    guest
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
}
