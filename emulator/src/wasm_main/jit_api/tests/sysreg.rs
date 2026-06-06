use super::super::sysreg::jit_read_sysreg_from_machine;
use crate::arm64::machine::Machine;
use crate::constants::{SYSREG_CNTVCT_EL0, SYSREG_ICC_IAR1_EL1, SYSREG_SP_EL0, SYSREG_TPIDR_EL0};

#[test]
fn jit_read_sysreg_reads_sp_el0() {
    let mut machine = Machine::new(1);
    machine.cpus[0].sys.sp_el0 = 0x1234_5678_9abc_def0;

    let value = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_SP_EL0)
        .expect("JIT sysreg helper should read SP_EL0");

    assert_eq!(value, 0x1234_5678_9abc_def0);
}

#[test]
fn jit_read_sysreg_reads_tpidr_el0() {
    let mut machine = Machine::new(1);
    machine.cpus[0].sys.tpidr_el0 = 0xfeed_face_cafe_beef;

    let value = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_TPIDR_EL0)
        .expect("JIT sysreg helper should read TPIDR_EL0");

    assert_eq!(value, 0xfeed_face_cafe_beef);
}

#[test]
fn jit_read_sysreg_reads_cntvct_el0() {
    let mut machine = Machine::new(1);
    machine.cpus[0].sys.cycle_count = 0x1234_5678;

    let value = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_CNTVCT_EL0)
        .expect("JIT sysreg helper should read CNTVCT_EL0");

    assert_eq!(value, 0x1234_5678);
}

#[test]
fn jit_read_sysreg_rejects_interrupt_acknowledge() {
    let mut machine = Machine::new(1);

    let err = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_ICC_IAR1_EL1)
        .expect_err("JIT sysreg helper must reject side-effectful reads");

    assert!(err.contains("0x4660"), "{err}");
}
