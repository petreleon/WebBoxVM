use crate::bus::SystemBus;
use super::layout::EFI_SUCCESS;

/// Encode ARM64 `MOVZ Xd, #imm16` (hw=0).
pub fn movz_x(d: u8, imm16: u16) -> u32 {
    let d = d as u32 & 0x1F;
    let imm = imm16 as u32;
    0xD280_0000 | (imm << 5) | d
}

/// Encode ARM64 `RET`.
pub const RET: u32 = 0xD65F_03C0;

/// Write 64-bit value to bus.
pub fn write64(bus: &mut SystemBus, addr: u64, val: u64) {
    bus.write(addr, 8, val);
}

/// Write 32-bit value to bus.
pub fn write32(bus: &mut SystemBus, addr: u64, val: u32) {
    bus.write(addr, 4, val as u64);
}

/// Write a minimal "return EFI_SUCCESS" trampoline at `addr` and return it.
pub fn write_success_trampoline(bus: &mut SystemBus, addr: u64, handle: u64) -> u64 {
    let imm16 = if handle == EFI_SUCCESS { 0 } else { (handle & 0xFFFF) as u16 };
    write32(bus, addr, movz_x(0, imm16));
    write32(bus, addr + 4, RET);
    addr
}
