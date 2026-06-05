use super::util;
use emulator::boot::BootContext;
use emulator::constants::INITRD_BASE;
use std::env;
use std::time::Instant;

pub(super) fn run_kernel_chunks(
    ctx: &mut BootContext,
    expected_initrd: Option<&[u8]>,
    t0: &Instant,
) {
    let mut last_uart = ctx.uart_output().len();
    let chunks = util::env_usize("BOOT_TEST_CHUNKS", 10);
    let per_chunk = util::env_usize("BOOT_TEST_STEPS", 2_000_000);
    let mut input = super::prompt_script::BootTestInput::from_env();
    let stop_uart_default = if input.disables_uart_stop() { 0 } else { 500 };
    let stop_uart = util::env_usize("BOOT_TEST_STOP_UART", stop_uart_default);
    let stop_text = env::var("BOOT_TEST_STOP_TEXT")
        .ok()
        .filter(|text| !text.is_empty());

    for i in 0..chunks {
        ctx.run_kernel_phase(per_chunk);
        maybe_report_initrd(ctx, expected_initrd);

        let uart = ctx.uart_output();
        let new_bytes = uart.len().saturating_sub(last_uart);
        report_uart_delta(ctx, &uart, last_uart, new_bytes, i, t0);
        last_uart = uart.len();

        input.maybe_feed(ctx, &uart);
        if should_stop(&uart, stop_text.as_deref(), stop_uart) {
            break;
        }
    }
}

fn maybe_report_initrd(ctx: &BootContext, expected_initrd: Option<&[u8]>) {
    if env::var_os("BOOT_TEST_CHECK_INITRD").is_none() {
        return;
    }
    let Some(expected) = expected_initrd else {
        return;
    };

    match util::first_initrd_mismatch(ctx, expected) {
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

fn report_uart_delta(
    ctx: &BootContext,
    uart: &str,
    last_uart: usize,
    new_bytes: usize,
    i: usize,
    t0: &Instant,
) {
    let elapsed = t0.elapsed().as_secs_f32();
    if new_bytes == 0 {
        println!(
            "Kernel chunk {}: 0B new UART in {:.1}s, PC=0x{:x}",
            i + 1,
            elapsed,
            ctx.pc()
        );
        if i > 0 {
            println!("  Warning: no new output for {}M steps", (i + 1) * 2);
        }
        return;
    }

    let start = last_uart.min(uart.len());
    let end = (start + new_bytes).min(uart.len());
    println!(
        "Kernel chunk {}: +{}B UART in {:.1}s, PC=0x{:x}, total UART={}B",
        i + 1,
        new_bytes,
        elapsed,
        ctx.pc(),
        uart.len()
    );
    print_uart_preview(&uart[start..end]);
}

fn print_uart_preview(preview: &str) {
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
}

fn should_stop(uart: &str, stop_text: Option<&str>, stop_uart: usize) -> bool {
    if let Some(text) = stop_text
        && uart.contains(text)
    {
        println!("Stop text matched: {text:?}");
        return true;
    }

    stop_uart > 0 && uart.len() > stop_uart
}
