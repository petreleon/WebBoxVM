use super::*;
use crate::efi::layout::{
    EFI_BOOT_SERVICES, EFI_LARGE_CODE, EFI_MEM_BASE, EFI_SERVICE_TRAMPOLINES, LARGE_CODE_STRIDE,
    TRAMPOLINE_STRIDE,
};
use std::collections::HashMap;

pub(super) fn is_efi(pc: u64) -> bool {
    (pc >= EFI_SERVICE_TRAMPOLINES && pc < EFI_SERVICE_TRAMPOLINES + 512 * TRAMPOLINE_STRIDE)
        || (pc >= EFI_LARGE_CODE && pc < EFI_LARGE_CODE + 16 * LARGE_CODE_STRIDE)
}

pub(super) fn build_fp_to_name(bus: &SystemBus) -> HashMap<u64, String> {
    let mut fp_to_name = HashMap::new();
    for &(off, name) in boot_service_offsets() {
        if let Some(fp) = bus.mem.read(EFI_BOOT_SERVICES + off, 8) {
            fp_to_name.insert(fp, name.to_string());
        }
    }
    let con_out_output_string = bus.mem.read(EFI_MEM_BASE + 0x6000 + 0x08, 8).unwrap_or(0);
    if con_out_output_string != 0 {
        fp_to_name.insert(con_out_output_string, "ConOut::OutputString".to_string());
    }
    fp_to_name
}

pub(super) fn efi_status(status: u64) -> &'static str {
    match status {
        0 => "EFI_SUCCESS",
        0x8000_0000_0000_0001 => "EFI_LOAD_ERROR",
        0x8000_0000_0000_0002 => "EFI_INVALID_PARAM",
        0x8000_0000_0000_0003 => "EFI_UNSUPPORTED",
        0x8000_0000_0000_0005 => "EFI_BUFFER_TOO_SMALL",
        0x8000_0000_0000_000E => "EFI_NOT_FOUND",
        _ => "UNKNOWN_STATUS",
    }
}

pub(super) fn print_trace_preamble(bus: &SystemBus, dtb_addr: u64, trace_steps: usize) {
    println!("DTB bytes at {:#x}:", dtb_addr);
    for i in 0..16u64 {
        print!("{:02x} ", bus.mem.read(dtb_addr + i, 1).unwrap_or(0xFF));
    }
    println!();

    let st = crate::efi::EFI_SYSTEM_TABLE;
    let config_table_ptr = bus.mem.read(st + 0x70, 8).unwrap_or(0);
    println!("ConfigurationTable pointer: {:#x}", config_table_ptr);
    if config_table_ptr != 0 {
        let g0 = bus.mem.read(config_table_ptr, 8).unwrap_or(0);
        let g1 = bus.mem.read(config_table_ptr + 8, 8).unwrap_or(0);
        let ptr = bus.mem.read(config_table_ptr + 16, 8).unwrap_or(0);
        println!("  GUID: {:#018x} {:#018x} -> table: {:#x}", g0, g1, ptr);
    }
    println!("Trace step limit: {}", trace_steps);
}

fn boot_service_offsets() -> &'static [(u64, &'static str)] {
    &[
        (0x18, "RaiseTPL"),
        (0x20, "RestoreTPL"),
        (0x28, "AllocatePages"),
        (0x30, "FreePages"),
        (0x38, "GetMemoryMap"),
        (0x40, "AllocatePool"),
        (0x48, "FreePool"),
        (0x50, "CreateEvent"),
        (0x58, "SetTimer"),
        (0x60, "WaitForEvent"),
        (0x68, "SignalEvent"),
        (0x70, "CloseEvent"),
        (0x78, "CheckEvent"),
        (0x80, "InstallProtocol"),
        (0x88, "ReinstallProto"),
        (0x90, "UninstallProto"),
        (0x98, "HandleProtocol"),
        (0xA0, "Reserved"),
        (0xA8, "RegisterProtoNotify"),
        (0xB0, "LocateHandle"),
        (0xB8, "LocateDevicePath"),
        (0xC0, "InstallConfigTable"),
        (0xC8, "LoadImage"),
        (0xD0, "StartImage"),
        (0xD8, "Exit"),
        (0xE0, "UnloadImage"),
        (0xE8, "ExitBootServices"),
        (0xF0, "GetNextMonotonicCount"),
        (0xF8, "Stall"),
        (0x100, "SetWatchdogTimer"),
        (0x108, "ConnectController"),
        (0x110, "DisconnectController"),
        (0x118, "OpenProtocol"),
        (0x120, "CloseProtocol"),
        (0x128, "OpenProtocolInfo"),
        (0x130, "ProtocolsPerHandle"),
        (0x138, "LocateHandleBuffer"),
        (0x140, "LocateProtocol"),
        (0x148, "InstallMultipleProtos"),
        (0x150, "UninstallMultipleProtos"),
        (0x158, "CalculateCrc32"),
        (0x160, "CopyMem"),
        (0x168, "SetMem"),
        (0x170, "CreateEventEx"),
    ]
}
