//! Long-running boot test — run until UART output appears or timeout.

use emulator::boot::BootContext;
use std::fs;
use std::time::Instant;

fn main() {
    let kernel = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").expect("read");
    let mut ctx = BootContext::new(&kernel, 1).expect("boot");
    let t0 = Instant::now();
    let mut last_uart = 0usize;
    let mut last_pc = 0u64;

    ctx.run_efi_phase(5_000_000);
    println!("EFI done: {}s PC=0x{:x}", t0.elapsed().as_secs(), ctx.pc());

    for i in 0..200 {
        ctx.run_kernel_phase(10_000_000);
        let uart = ctx.uart_output();
        let pc = ctx.pc();

        if uart.len() > last_uart {
            let new = &uart[last_uart..];
            println!("\n!!! UART +{}B at step {}M ({:.0}s) !!!", new.len(), (i+1)*10, t0.elapsed().as_secs_f64());
            println!("{}", new.chars().take(2000).collect::<String>());
            last_uart = uart.len();
        }

        if pc != last_pc {
            let range = if pc >= 0xffff000000000000 { "KVA" } else { "phys" };
            println!("  {}0M steps {:.0}s PC=0x{:x} [{}]", i+1, t0.elapsed().as_secs_f64(), pc, range);
            last_pc = pc;
        }

        if uart.len() > 500 { break; }
        if i > 0 && i % 50 == 0 {
            println!("  ... {}M steps, {}s elapsed, no UART yet", (i+1)*10, t0.elapsed().as_secs());
        }
    }

    let u = ctx.uart_output();
    println!("\nFinal: {} steps {:.0}s UART={}B", ctx.total_steps(), t0.elapsed().as_secs_f64(), u.len());
}
