use super::*;

pub(in crate::arch::arm64::execute) fn translate_or_data_fault(
    cpu: &mut Armv8Cpu,
    mem: &mut crate::memory::PhysicalMemory,
    va: u64,
    write: bool,
    _err: &'static str,
) -> Result<u64, &'static str> {
    let result = if write {
        translate_write(&cpu.sys, &mut cpu.tlb, mem, va, cpu.pstate.el())
    } else {
        translate(&cpu.sys, &mut cpu.tlb, mem, va)
    };

    match result {
        Ok(pa) => Ok(pa),
        Err(
            fault @ (Fault::TranslationFault | Fault::AccessFlagFault | Fault::PermissionFault),
        ) => {
            cpu.sys.far_el1 = va;
            Err(match fault {
                Fault::TranslationFault => "translation fault",
                Fault::AccessFlagFault => "access flag fault",
                Fault::PermissionFault => "permission fault",
            })
        }
    }
}
