use super::*;

pub(in crate::arm64::execute) fn set_fp_compare_flags(cpu: &mut Armv8Cpu, lhs: f64, rhs: f64) {
    if lhs.is_nan() || rhs.is_nan() {
        cpu.pstate.set_nzcv(false, false, true, true);
    } else if lhs == rhs {
        cpu.pstate.set_nzcv(false, true, true, false);
    } else if lhs < rhs {
        cpu.pstate.set_nzcv(true, false, false, false);
    } else {
        cpu.pstate.set_nzcv(false, false, true, false);
    }
}

pub(in crate::arm64::execute) fn set_nzcv_from_bits(cpu: &mut Armv8Cpu, bits: u64) {
    cpu.pstate.set_nzcv(
        (bits & 8) != 0,
        (bits & 4) != 0,
        (bits & 2) != 0,
        (bits & 1) != 0,
    );
}
