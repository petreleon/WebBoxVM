use super::*;

pub(in crate::arm64::machine) fn deliver_external_irq(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    if cpu.sys.vbar_el1 == 0 || cpu.sys.irq_pending || cpu.pstate.irq_masked() {
        return;
    }

    let Some(int_id) = bus.gic.next_pending_enabled() else {
        return;
    };

    cpu.sys.irq_pending = true;
    cpu.sys.last_irq_id = int_id;
    cpu.clear_exclusive();
    let from_lower_el = cpu.pstate.el() == 0;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.elr_el1 = cpu.regs.pc;
    cpu.sys.esr_el1 = 0;
    cpu.enter_el1_exception(from_lower_el);
    cpu.regs.pc = cpu.sys.vbar_el1
        + if from_lower_el {
            VBAR_IRQ_LOWER_EL_AARCH64
        } else {
            VBAR_IRQ_CURRENT_EL
        };
}

pub(in crate::arm64::machine) fn take_instruction_abort(cpu: &mut Armv8Cpu, fault_pc: u64) {
    let from_lower_el = cpu.pstate.el() == 0;
    let ec = if from_lower_el {
        ESR_EC_INSN_ABORT_LOWER_EL
    } else {
        ESR_EC_INSN_ABORT_CURRENT_EL
    };
    take_sync_exception(cpu, fault_pc, ec, ESR_FSC_TRANSLATION_LEVEL3, from_lower_el);
}

pub(in crate::arm64::machine) fn take_data_abort(
    cpu: &mut Armv8Cpu,
    fault_pc: u64,
    instr: Instr,
    err: &str,
    trace_el0_faults: bool,
) {
    let from_lower_el = cpu.pstate.el() == 0;
    let ec = if from_lower_el {
        ESR_EC_DATA_ABORT_LOWER_EL
    } else {
        ESR_EC_DATA_ABORT_CURRENT_EL
    };
    let fsc = data_abort_fsc(err);
    let iss = fsc
        | if memory_fault_is_write(instr) {
            ESR_DATA_ABORT_WNR
        } else {
            0
        };
    if from_lower_el && trace_el0_faults {
        trace_el0_data_abort(cpu, fault_pc, instr, iss);
    }
    take_sync_exception(cpu, fault_pc, ec, iss, from_lower_el);
}

pub(in crate::arm64::machine) fn take_fp_simd_trap(cpu: &mut Armv8Cpu, fault_pc: u64) {
    let from_lower_el = cpu.pstate.el() == 0;
    take_sync_exception(
        cpu,
        fault_pc,
        ESR_EC_FP_ASIMD,
        ESR_FP_ASIMD_ISS_AARCH64,
        from_lower_el,
    );
}

pub(in crate::arm64::machine) fn is_data_abort_fault(err: &str) -> bool {
    err.contains("translation fault")
        || err.contains("permission fault")
        || err.contains("access flag fault")
}

fn data_abort_fsc(err: &str) -> u64 {
    if err.contains("permission fault") {
        ESR_FSC_PERMISSION_LEVEL3
    } else if err.contains("access flag fault") {
        ESR_FSC_ACCESS_FLAG_LEVEL3
    } else {
        ESR_FSC_TRANSLATION_LEVEL3
    }
}

fn trace_el0_data_abort(cpu: &Armv8Cpu, fault_pc: u64, instr: Instr, iss: u64) {
    eprintln!(
        "EL0 DATA ABORT pc=0x{fault_pc:016x} instr={instr:?} far=0x{:016x} iss=0x{iss:x} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
         x8=0x{:016x} sp=0x{:016x} sp_el0=0x{:016x} sp_el1=0x{:016x} elr=0x{:016x} spsr=0x{:x}",
        cpu.sys.far_el1,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(8),
        cpu.regs.sp,
        cpu.sys.sp_el0,
        cpu.sys.sp_el1,
        cpu.sys.elr_el1,
        cpu.sys.spsr_el1,
    );
}

fn take_sync_exception(cpu: &mut Armv8Cpu, fault_pc: u64, ec: u64, iss: u64, from_lower_el: bool) {
    cpu.clear_exclusive();
    cpu.sys.elr_el1 = fault_pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = (ec << 26) | iss;
    cpu.enter_el1_exception(from_lower_el);
    let vector = if from_lower_el {
        VBAR_SYNC_LOWER_EL_AARCH64
    } else {
        VBAR_SYNC_CURRENT_EL
    };
    cpu.regs.pc = cpu.sys.vbar_el1 + vector;
}

fn memory_fault_is_write(instr: Instr) -> bool {
    matches!(
        instr.op,
        Opcode::Str
            | Opcode::Stp
            | Opcode::SimdStr
            | Opcode::SimdStp
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt1Lane
            | Opcode::SimdSt4Single
            | Opcode::SimdSt4
            | Opcode::Stxr
            | Opcode::Stlr
            | Opcode::Stxp
            | Opcode::Atomic
            | Opcode::AtomicPair
            | Opcode::Cas
            | Opcode::Casp
            | Opcode::DcZva
    )
}
