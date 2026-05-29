// ── EFI stub trap handlers ──

use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::constants::*;

/// Returns true if PC matched an EFI trap address and was handled.
pub(super) fn handle_efi_service_trap(cpu: &mut Armv8Cpu, bus: &mut SystemBus, pages_bump: &mut u64) -> bool {
    match cpu.regs.pc {
        EFI_TRAP_COPYMEM => {
            // CopyMem(Dest=X0, Src=X1, Length=X2)
            let dest = cpu.regs.x(0);
            let src  = cpu.regs.x(1);
            let len  = cpu.regs.x(2);
            if len > EFI_MAX_COPY_SIZE { return false; }
            for i in 0..len {
                if let Some(val) = bus.mem.read(src + i, 1) {
                    bus.mem.write(dest + i, 1, val);
                }
            }
            cpu.regs.set_x(0, EFI_SUCCESS);
            cpu.regs.pc = cpu.regs.x(LINK_REGISTER_INDEX);
            true
        }
        EFI_TRAP_SETMEM => {
            // SetMem(Buffer=X0, Size=X1, Value=X2)
            let buf  = cpu.regs.x(0);
            let size = cpu.regs.x(1);
            let val  = cpu.regs.x(2);
            if size > EFI_MAX_COPY_SIZE { return false; }
            for i in 0..size { bus.mem.write(buf + i, 1, val); }
            cpu.regs.set_x(0, EFI_SUCCESS);
            cpu.regs.pc = cpu.regs.x(LINK_REGISTER_INDEX);
            true
        }
        _ => false,
    }
}

/// Returns true if the current PC is a known cache-invalidation loop (fast-forward it).
pub(super) fn handle_cache_loop_fast_forward(cpu: &mut Armv8Cpu) -> bool {
    match cpu.regs.pc {
        CACHE_INV_LOOP_ENTRY => {
            // DC CIVAC loop: set counter to range end so SUB/CMP finishes
            cpu.regs.set_x(2, cpu.regs.x(3));
            cpu.pstate.set_nzcv(false, true, true, false); // N=0, Z=1, C=1, V=0 → EQ+CS
            cpu.regs.pc = CACHE_INV_LOOP_EXIT;
            true
        }
        I_CACHE_INV_LOOP_ENTRY => {
            // IC IVAU loop
            cpu.regs.set_x(3, cpu.regs.x(1));
            cpu.pstate.set_nzcv(false, true, true, false);
            cpu.regs.pc = I_CACHE_INV_LOOP_EXIT;
            true
        }
        _ => false,
    }
}
