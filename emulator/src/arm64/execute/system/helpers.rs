use super::*;

pub(super) fn fault_to_error(fault: Fault) -> &'static str {
    match fault {
        Fault::TranslationFault => "translation fault",
        Fault::AccessFlagFault => "access flag fault",
        Fault::PermissionFault => "permission fault",
    }
}

pub(super) fn trace_daif(cpu: &Armv8Cpu, label: &str) {
    if env::var_os("WEBBOXVM_TRACE_DAIF").is_some() {
        eprintln!(
            "DAIF {label} pc=0x{:016x} pstate=0x{:x} spsr=0x{:x} elr=0x{:016x} irq_masked={}",
            cpu.regs.pc,
            cpu.pstate.to_u64(),
            cpu.sys.spsr_el1,
            cpu.sys.elr_el1,
            cpu.pstate.irq_masked()
        );
    }
}
