//! Long-running boot test — prints progress and exits when UART output appears.
//! Run: cargo run --example wait_uart --release

use emulator::boot::BootContext;
use std::fs;
use std::time::Instant;

fn main() {
    let kernel = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").expect("read Image.gz");
    let mut ctx = BootContext::new(&kernel, 1).expect("boot");
    println!("Boot context ready");

    ctx.run_efi_phase(5_000_000);
    println!("EFI done, entering kernel...");

    let t0 = Instant::now();
    let mut last_uart = 0usize;
    let mut step = 0u64;

    loop {
        ctx.run_kernel_phase(10_000_000);
        step += 10_000_000;
        let uart = ctx.uart_output();

        if uart.len() > last_uart {
            let new = &uart[last_uart..];
            println!("\n=== UART at {}M steps ({:.0}s) ===", step / 1_000_000, t0.elapsed().as_secs_f64());
            println!("{}", new);
            last_uart = uart.len();
            if uart.len() > 200 { break; }
        }

        let elapsed = t0.elapsed().as_secs();
        if step % 100_000_000 == 0 {
            println!("{}M steps, {}s, no UART yet", step / 1_000_000, elapsed);
        }

        if step >= 2_000_000_000 { println!("2B steps reached, giving up"); break; }
    }
}
