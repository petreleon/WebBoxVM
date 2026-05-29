use crate::bus::SystemBus;
use crate::constants::*;

/// Encode ARM64 `MOVZ Xd, #imm16` (hw=0, LSL #0).
pub fn movz_x(rd: u8, imm16: u16) -> u32 {
    let rd_bits = (rd as u32) & 0x1F;
    MOVZ_BASE | ((imm16 as u32) << 5) | rd_bits
}

/// Encode ARM64 `MOVK Xd, #imm16, LSL #(16*hw)`.
pub fn movk_x(rd: u8, hw: u8, imm16: u16) -> u32 {
    let rd_bits = (rd as u32) & 0x1F;
    let hw_bits = ((hw as u32) & 3) << 21;
    MOVK_BASE | hw_bits | ((imm16 as u32) << 5) | rd_bits
}

/// Write a 64-bit value to the system bus at the given address.
pub fn write64(bus: &mut SystemBus, addr: u64, val: u64) {
    bus.write(addr, 8, val);
}

/// Write a 32-bit value to the system bus at the given address.
pub fn write32(bus: &mut SystemBus, addr: u64, val: u32) {
    bus.write(addr, 4, val as u64);
}

/// Write a minimal "return EFI_SUCCESS" trampoline at `addr`.
/// Consists of: MOVZ X0, #0 (or #handle_low) ; RET.
/// Returns the trampoline address (same as `addr`).
pub fn write_success_trampoline(bus: &mut SystemBus, addr: u64, handle: u64) -> u64 {
    let imm16 = if handle == EFI_SUCCESS { 0 } else { (handle & 0xFFFF) as u16 };
    write32(bus, addr, movz_x(0, imm16));
    write32(bus, addr + INSTRUCTION_SIZE, INSTR_RET);
    addr
}
