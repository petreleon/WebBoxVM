use emulator::boot::BootContext;
use std::fs;
use std::time::Instant;

fn main() {
    let kernel = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").expect("read");
    let mut ctx = BootContext::new(&kernel, 1).expect("boot");
    ctx.run_efi_phase(5_000_000);
    let cpu = &ctx.machine.cpus[0];
    println!("After EFI: vbar=0x{:x} el={} irq_masked={}", 
        cpu.sys.vbar_el1, cpu.pstate.el(), cpu.pstate.irq_masked());
    println!("           sctlr=0x{:x} ttbr1=0x{:x}", cpu.sys.sctlr_el1, cpu.sys.ttbr1_el1);
    
    let t0 = Instant::now();
    for round in 0..30 {
        ctx.run_kernel_phase(1_000_000);
        let cpu = &ctx.machine.cpus[0];
        let uart = ctx.uart_output();
        let pc = cpu.regs.pc;
        
        if pc >= 0xffff000000000000 {
            println!("Kernel {}M: PC=0x{:x} vbar=0x{:x} el={} irq_off={} cycle=0x{:x} cntp_ctl=0x{:x} cntp_cval=0x{:x}",
                (round+1), pc, cpu.sys.vbar_el1, cpu.pstate.el(), 
                cpu.pstate.irq_masked(), cpu.sys.cycle_count,
                cpu.sys.cntp_ctl_el0, cpu.sys.cntp_cval_el0);
        }
        if uart.len() > 50 { println!("UART: {}", uart); break; }
        if (round+1) % 10 == 0 { println!("  {}M steps, no UART yet", (round+1)); }
    }
}
