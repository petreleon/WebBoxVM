use super::*;
use std::env;

pub(super) fn trace_write_permission(
    sys: &SystemRegisters,
    walk: walk::WalkResult,
    va: u64,
    current_el: u8,
) {
    if env::var_os("WEBBOXVM_TRACE_WRITE_PERM").is_none() {
        return;
    }

    let read_only = (walk.desc & DESC_AP_RO) != 0;
    let el0_accessible = (walk.desc & DESC_AP_EL0) != 0;
    if !read_only && (current_el != 0 || el0_accessible) {
        return;
    }

    eprintln!(
        "WRITE PERM va=0x{va:016x} pa=0x{:016x} desc_addr=0x{:016x} desc=0x{:016x} tcr=0x{:016x} el={current_el} ro={} el0={} dbm={} ha={} hd={}",
        walk.pa,
        walk.desc_addr,
        walk.desc,
        sys.tcr_el1,
        read_only,
        el0_accessible,
        (walk.desc & DESC_DBM_BIT) != 0,
        (sys.tcr_el1 & TCR_HA_BIT) != 0,
        (sys.tcr_el1 & TCR_HD_BIT) != 0,
    );
}

pub(super) fn check_write_permission(
    sys: &SystemRegisters,
    mem: &mut PhysicalMemory,
    desc_addr: u64,
    desc: u64,
    current_el: u8,
) -> Result<(), Fault> {
    let read_only = (desc & DESC_AP_RO) != 0;
    let el0_accessible = (desc & DESC_AP_EL0) != 0;

    if current_el == 0 && !el0_accessible {
        return Err(Fault::PermissionFault);
    }

    if read_only {
        if can_hardware_update_dirty(sys, desc) {
            mark_descriptor_dirty(mem, desc_addr, desc)?;
            return Ok(());
        }
        return Err(Fault::PermissionFault);
    }

    Ok(())
}

fn can_hardware_update_dirty(sys: &SystemRegisters, desc: u64) -> bool {
    hardware_dirty_update_enabled(sys) && (desc & DESC_DBM_BIT) != 0
}

fn hardware_dirty_update_enabled(sys: &SystemRegisters) -> bool {
    (sys.tcr_el1 & (TCR_HA_BIT | TCR_HD_BIT)) == (TCR_HA_BIT | TCR_HD_BIT)
}

fn mark_descriptor_dirty(mem: &mut PhysicalMemory, desc_addr: u64, desc: u64) -> Result<(), Fault> {
    let dirty_desc = desc & !DESC_AP_RO;
    mem.write(desc_addr, 8, dirty_desc)
        .ok_or(Fault::TranslationFault)
}
