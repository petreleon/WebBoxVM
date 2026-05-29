use emulator::boot::BootContext;
use emulator::arm64::{decode, translate, Tlb};
use std::fs;

fn main() {
    let kernel = fs::read("/Users/petreleon/code/WebBoxVM/Image.gz").expect("read");
    let mut ctx = BootContext::new(&kernel, 1).expect("boot");
    ctx.run_efi_phase(5_000_000);

    let pc = ctx.machine.cpus[0].regs.pc;
    println!("After EFI: vbar=0x{:x} el={} ttbr1=0x{:x} sctlr=0x{:x}",
        ctx.machine.cpus[0].sys.vbar_el1, ctx.machine.cpus[0].pstate.el(),
        ctx.machine.cpus[0].sys.ttbr1_el1, ctx.machine.cpus[0].sys.sctlr_el1);

    for round in 0..5 {
        ctx.run_kernel_phase(1_000_000);
        let pc = ctx.machine.cpus[0].regs.pc;
        if pc >= 0xffff000000000000 && round >= 3 {
            let mut scratch = Tlb::new();
            let sys = &ctx.machine.cpus[0].sys;
            let mem = &ctx.machine.bus.mem;
            println!("\n=== Round {}: PC=0x{:x} ===", round, pc);
            for &va in &[pc, pc+4, pc-4, 0xffff8000800a3240u64] {
                match translate(sys, &mut scratch, mem, va) {
                    Ok(pa) => {
                        if let Some(raw) = mem.read(pa, 4) {
                            if let Some(instr) = decode(raw as u32) {
                                println!("  VA 0x{:016x} -> PA 0x{:016x}: {:?}", va, pa, instr.op);
                            } else {
                                println!("  VA 0x{:016x} -> PA 0x{:016x}: 0x{:08x} ???", va, pa, raw as u32);
                            }
                        } else {
                            println!("  VA 0x{:016x} -> PA 0x{:016x}: BUS FAULT", va, pa);
                        }
                    }
                    Err(f) => println!("  VA 0x{:016x}: FAULT {:?}", va, f),
                }
            }
            break;
        }
    }
}
