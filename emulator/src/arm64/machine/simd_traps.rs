use super::*;

pub(in crate::arm64::machine) fn fp_simd_access_traps(cpu: &Armv8Cpu) -> bool {
    let fpen = (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT;
    match cpu.pstate.el() {
        0 => fpen != CPACR_FPEN_TRAP_NONE,
        1 => matches!(fpen, CPACR_FPEN_TRAP_EL0_EL1 | CPACR_FPEN_TRAP_EL1_EL0),
        _ => false,
    }
}

pub(in crate::arm64::machine) fn is_fp_simd_access(instr: Instr) -> bool {
    match instr.op {
        Opcode::Mrs | Opcode::Msr => {
            let sysreg_id = instr.imm as u16;
            matches!(sysreg_id, SYSREG_FPCR | SYSREG_FPSR)
        }
        op => is_fp_simd_opcode(op),
    }
}

fn is_fp_simd_opcode(op: Opcode) -> bool {
    is_sve_opcode(op)
        || is_simd_memory_or_crypto_opcode(op)
        || is_simd_integer_opcode(op)
        || is_fp_scalar_opcode(op)
}
