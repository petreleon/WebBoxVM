use super::*;

pub(super) enum KernelShortcut {
    None,
    Handled,
    Relocation(usize),
}

pub(super) fn handle_kernel_shortcut(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    pages_bump: &mut u64,
    steps: &mut u64,
) -> KernelShortcut {
    if cpu.regs.pc == 0x8000_CE00 {
        let dest = cpu.regs.x(0);
        let src = cpu.regs.x(1);
        let mut len = cpu.regs.x(2);
        if len > 0x0400_0000 {
            println!("WARNING: CopyMem with huge len = {:#x}, capping to 0", len);
            len = 0;
        }
        if dest > src && src + len > dest {
            for i in (0..len).rev() {
                let val = bus.mem.read(src + i, 1).unwrap_or(0);
                bus.mem.write(dest + i, 1, val);
            }
        } else {
            for i in 0..len {
                let val = bus.mem.read(src + i, 1).unwrap_or(0);
                bus.mem.write(dest + i, 1, val);
            }
        }
        return efi_success_return(cpu, steps);
    }

    if cpu.regs.pc == 0x8000_D000 {
        let buf = cpu.regs.x(0);
        let mut size = cpu.regs.x(1);
        let val = cpu.regs.x(2);
        if size > 0x0400_0000 {
            println!("WARNING: SetMem with huge size = {:#x}, capping to 0", size);
            size = 0;
        }
        for i in 0..size {
            bus.mem.write(buf + i, 1, val);
        }
        return efi_success_return(cpu, steps);
    }

    if cpu.regs.pc == 0x8000_D200 {
        let pages = cpu.regs.x(2);
        let ptr_memory = cpu.regs.x(3);
        let allocated = (*pages_bump + 4095) & !4095;
        *pages_bump = allocated + pages * 4096;
        bus.write(ptr_memory, 8, allocated);
        return efi_success_return(cpu, steps);
    }

    if cpu.regs.pc == 0x8000_D400 {
        return efi_success_return(cpu, steps);
    }

    if cpu.regs.pc == 0x400b6e80 {
        let x3 = cpu.regs.x(3);
        cpu.regs.set_x(2, x3);
        cpu.pstate.set_nzcv(false, true, true, false);
        cpu.regs.pc = 0x400b6e90;
        return KernelShortcut::Handled;
    }

    if cpu.regs.pc == 0x400b6eb8 {
        let x1 = cpu.regs.x(1);
        cpu.regs.set_x(3, x1);
        cpu.pstate.set_nzcv(false, true, true, false);
        cpu.regs.pc = 0x400b6ec8;
        return KernelShortcut::Handled;
    }

    if let Some(records) = fast_forward_efi_relocation_loop(cpu, bus) {
        return KernelShortcut::Relocation(records);
    }

    KernelShortcut::None
}

fn efi_success_return(cpu: &mut Armv8Cpu, steps: &mut u64) -> KernelShortcut {
    cpu.regs.set_x(0, 0);
    cpu.regs.pc = cpu.regs.x(30);
    *steps += 1;
    KernelShortcut::Handled
}
