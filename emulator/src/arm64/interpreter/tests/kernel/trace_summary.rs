use super::*;

impl EfiTrace {
    pub(super) fn print_summary(&self, cpu: &Armv8Cpu, bus: &SystemBus, steps: u64) {
        println!("\n=== EFI Call Log ({} calls) ===", self.log.len());
        for line in &self.log {
            println!("{}", line);
        }
        println!("\n=== Final State ({} steps) ===", steps);
        println!("  PC={:#016x}  SP={:#016x}", cpu.regs.pc, cpu.regs.sp);
        println!("  X0={:#018x}  X1={:#018x}", cpu.regs.x(0), cpu.regs.x(1));
        println!(
            "  X19={:#018x} X20={:#018x} X21={:#018x}",
            cpu.regs.x(19),
            cpu.regs.x(20),
            cpu.regs.x(21)
        );
        println!("  UART: {:?}", bus.uart.output_string());
        println!("  Trace -> /tmp/kernel_trace.txt");
        println!("\n--- Last 80 from ring ---");
        self.print_recent(80);
    }

    pub(super) fn print_recent(&self, count: usize) {
        for line in self
            .recent
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .iter()
            .rev()
        {
            println!("{}", line);
        }
    }
}
