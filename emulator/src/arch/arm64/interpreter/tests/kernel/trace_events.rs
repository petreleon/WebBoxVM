use super::*;
use std::io::Write;

impl EfiTrace {
    pub(super) fn observe_call_and_return(
        &mut self,
        steps: u64,
        cpu: &Armv8Cpu,
        bus: &SystemBus,
        instr: &crate::arch::arm64::opcodes::Instr,
    ) {
        if let Some(tgt) = efi_call_target(cpu, instr) {
            self.record_call(steps, cpu, bus, tgt);
        }
        if matches!(instr.op, Opcode::Ret) && is_efi(cpu.regs.pc) {
            self.record_return(steps, cpu, bus);
        }
    }

    pub(super) fn record_instruction(
        &mut self,
        steps: u64,
        cpu: &Armv8Cpu,
        instr: &crate::arch::arm64::opcodes::Instr,
    ) {
        if let Some(ref mut f) = self.file {
            let _ = writeln!(
                f,
                "{:7} {:#016x} {:?} X0={:#018x} X19={:#018x} SP={:#016x}",
                steps,
                cpu.regs.pc,
                instr.op,
                cpu.regs.x(0),
                cpu.regs.x(19),
                cpu.regs.sp
            );
        }
        self.recent
            .push_back(format!("{:7} {:#016x} {:?}", steps, cpu.regs.pc, instr.op));
        if self.recent.len() > 120 {
            self.recent.pop_front();
        }
    }

    fn record_call(&mut self, steps: u64, cpu: &Armv8Cpu, bus: &SystemBus, tgt: u64) {
        if !is_efi(tgt) {
            return;
        }
        let name = self.resolve(tgt);
        self.stack.push((
            cpu.regs.pc,
            tgt,
            cpu.regs.x(0),
            cpu.regs.x(1),
            cpu.regs.x(2),
            cpu.regs.x(3),
        ));
        let mut s = format!(
            "[{:7}] CALL {:<32} caller={:#x} X0={:#x} X1={:#x} X2={:#x} X3={:#x}",
            steps,
            name,
            cpu.regs.pc,
            cpu.regs.x(0),
            cpu.regs.x(1),
            cpu.regs.x(2),
            cpu.regs.x(3)
        );
        if name == "ConOut::OutputString" {
            s = format!("{} STR={:?}", s, read_utf16_string(bus, cpu.regs.x(1)));
        }
        println!("{}", s);
        self.log.push(s);
    }

    fn record_return(&mut self, steps: u64, cpu: &Armv8Cpu, bus: &SystemBus) {
        if let Some((caller, entry, ax0, ax1, ax2, ax3)) = self.stack.pop() {
            let name = self.resolve(entry);
            let r0 = cpu.regs.x(0);
            let status = efi_status(r0);
            let s = format!("[{:7}] RET  {:<32} -> {} ({:#x})", steps, name, status, r0);
            println!("{}", s);
            self.log.push(s);
            if r0 == 0x8000_0000_0000_0001 {
                print_load_error_context(bus, &name, caller, ax0, ax1, ax2, ax3);
            }
        }
    }
}

fn efi_call_target(cpu: &Armv8Cpu, instr: &crate::arch::arm64::opcodes::Instr) -> Option<u64> {
    match instr.op {
        Opcode::Blr => Some(cpu.regs.x(instr.rn)),
        Opcode::Bl => Some((cpu.regs.pc as i64 + instr.imm as i64) as u64),
        _ => None,
    }
}

fn read_utf16_string(bus: &SystemBus, mut addr: u64) -> String {
    let mut utf16_str = String::new();
    loop {
        let ch = bus.mem.read(addr, 2).unwrap_or(0);
        if ch == 0 {
            break;
        }
        if let Some(c) = std::char::from_u32(ch as u32) {
            utf16_str.push(c);
        }
        addr += 2;
    }
    utf16_str
}

fn print_load_error_context(
    bus: &SystemBus,
    name: &str,
    caller: u64,
    ax0: u64,
    ax1: u64,
    ax2: u64,
    ax3: u64,
) {
    println!("!!! FIRST EFI_LOAD_ERROR: {} (caller={:#x})", name, caller);
    println!(
        "    Args: X0={:#x} X1={:#x} X2={:#x} X3={:#x}",
        ax0, ax1, ax2, ax3
    );
    if ax1 >= 0x1000 && ax1 < 0x1_0000_0000 {
        print!("    [X1]= ");
        for i in 0..16u64 {
            print!("{:02x} ", bus.mem.read(ax1 + i, 1).unwrap_or(0xff) as u8);
        }
        println!();
    }
}
