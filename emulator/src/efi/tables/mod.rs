use super::encode::{movk_x, movz_x, write32, write64};
use super::protocols::loaded_image_protocol_addr;
use crate::constants::*;
use crate::platform::virt::SystemBus;

mod large_blocks;
mod services;
mod trampolines;

/// Encode a `RET X30` instruction.
pub(super) fn encode_ret() -> u32 {
    INSTR_RET
}

/// Write a sequence of 32-bit ARM64 instructions to memory.
pub(super) fn write_trampoline(bus: &mut SystemBus, addr: u64, insts: &[u32]) {
    for (i, &inst) in insts.iter().enumerate() {
        write32(bus, addr + (i as u64 * INSTRUCTION_SIZE), inst);
    }
}

/// Encode the bump-allocator trampoline (AllocatePool).
///
/// Allocates memory from a linear pool by bumping a head pointer.
///   X0 ← size (in bytes)
///   X2 ← **Buffer (where to write the pointer)
///   Returns EFI_SUCCESS (0).
fn bump_allocator_trampoline(head_ptr: u64) -> [u32; 8] {
    [
        movz_x(4, (head_ptr & 0xFFFF) as u16),
        movk_x(4, 1, ((head_ptr >> 16) & 0xFFFF) as u16),
        0xF9400085,   // LDR  X5, [X4]      // read current pool head
        0x8B0100A0,   // ADD  X0, X5, X1    // X0 = head + size (X1=Size)
        0xF9000080,   // STR  X0, [X4]      // update pool head
        0xF9000045,   // STR  X5, [X2]      // *Buffer = old head (X2=**Buffer)
        movz_x(0, 0), // MOVZ X0, #0        // EFI_SUCCESS
        encode_ret(),
    ]
}

/// Build a MOVZ/MOVK sequence to materialize a 64-bit constant in register `rd`.
pub(super) fn encode_mov64(insts: &mut Vec<u32>, rd: u8, val: u64) {
    insts.push(movz_x(rd, (val & 0xFFFF) as u16));
    if val >> 16 != 0 {
        insts.push(movk_x(rd, 1, ((val >> 16) & 0xFFFF) as u16));
    }
    if val >> 32 != 0 {
        insts.push(movk_x(rd, 2, ((val >> 32) & 0xFFFF) as u16));
    }
    if val >> 48 != 0 {
        insts.push(movk_x(rd, 3, ((val >> 48) & 0xFFFF) as u16));
    }
}

/// Build and install all UEFI firmware structures into the system bus.
pub fn setup_efi_tables(
    bus: &mut SystemBus,
    image_base: u64,
    image_size: u64,
    dtb_addr: u64,
) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;
    write64(bus, handle, 0x1_0000);
    write64(bus, EFI_SYSTEM_TABLE_PTR_ADDR, EFI_SYSTEM_TABLE_ADDR);

    let st = EFI_SYSTEM_TABLE_ADDR;
    write_system_table_header(bus, st);
    let con_out_struct = install_console_and_config_table(bus, st, dtb_addr);
    services::install_default_services(bus);
    large_blocks::install_specialized_trampolines(bus, con_out_struct);

    write64(bus, BOOT_SERVICES_VPTR_ADDR, EFI_BOOT_SERVICES_ADDR);
    super::protocols::install_loaded_image_protocol(bus, image_base, image_size);
    write64(bus, EFI_IMAGE_INFO_ADDR, image_base);
    write64(bus, EFI_IMAGE_INFO_ADDR + 8, image_size);

    (handle, st)
}

fn write_system_table_header(bus: &mut SystemBus, st: u64) {
    write64(bus, st + 0x00, EFI_ST_SIGNATURE);
    write32(bus, st + 0x08, EFI_ST_REVISION);
    write32(bus, st + 0x0C, EFI_ST_HEADER_SIZE);
    write32(bus, st + 0x10, 0);
    write32(bus, st + 0x14, 0);
}

fn install_console_and_config_table(bus: &mut SystemBus, st: u64, dtb_addr: u64) -> u64 {
    let con_out_struct = CONSOLE_OUT_STRUCT_ADDR;
    let con_out_handle = CONSOLE_OUT_HANDLE_ADDR;
    write64(bus, st + 0x18, 0);
    write32(bus, st + 0x20, 0);
    write32(bus, st + 0x24, 0);
    write64(bus, st + 0x28, 0);
    write64(bus, st + 0x30, 0);
    write64(bus, st + 0x38, con_out_handle);
    write64(bus, st + 0x40, con_out_struct);
    write64(bus, st + 0x48, con_out_handle);
    write64(bus, st + 0x50, con_out_struct);
    write64(bus, st + 0x58, EFI_RUNTIME_SERVICES_ADDR);
    write64(bus, st + 0x60, EFI_BOOT_SERVICES_ADDR);

    let config_table = EFI_CONFIG_TABLE_ADDR;
    write64(bus, config_table + 0, 0x41a5_f19c_b1b6_21d5);
    write64(bus, config_table + 8, 0xe0aa_692c_15d9_0b83);
    write64(bus, config_table + 16, dtb_addr);
    write64(bus, st + 0x68, EFI_MEMORY_DESC_COUNT);
    write64(bus, st + 0x70, config_table);
    con_out_struct
}
