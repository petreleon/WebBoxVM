//! Long-running boot test — prints progress and exits when UART output appears.
//! Run: cargo run --example wait_uart --release

use emulator::boot::BootContext;
use std::fs;
use std::time::Instant;

fn main() {
    let kernel = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").expect("read Image.gz");
    let mut ctx = BootContext::new(&kernel, 1).expect("boot");
    println!("Boot context ready");

    ctx.run_efi_phase(50_000_000);
    println!("EFI done, entering kernel...");

    let t0 = Instant::now();
    let mut last_uart = 0usize;
    let mut step = 0u64;

    let mut last_report = 0u64;
    loop {
        ctx.run_kernel_phase(1_000_000); // smaller chunks for finer-grained checks
        step += 1_000_000;
        let uart = ctx.uart_output();
        let elapsed = t0.elapsed().as_secs_f64();
        let msteps = step / 1_000_000;

        if uart.len() > last_uart {
            let new = &uart[last_uart..];
            println!("\n=== UART at {}M steps ({:.0}s) ===", msteps, elapsed);
            println!("{}", new);
            last_uart = uart.len();
            if uart.len() > 200 { break; }
        }

        // Print progress every 10M steps
        if step - last_report >= 10_000_000 || msteps <= 100 {
            println!("{}M steps, {:.0}s, PC=0x{:016x}, UART={}B, faults=({},{})",
                msteps, elapsed, ctx.pc(), uart.len(),
                ctx.machine.fetch_faults, ctx.machine.exec_faults);
            last_report = step;
        }
    }
}
