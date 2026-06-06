use super::super::sysreg::jit_read_sysreg_from_machine;
use crate::arm64::machine::Machine;
use crate::constants::{
    PSTATE_DAIF_MASK, SYSREG_CNTVCT_EL0, SYSREG_DAIF, SYSREG_ICC_IAR1_EL1, SYSREG_SP_EL0,
    SYSREG_TCR_EL1, SYSREG_TPIDR_EL0, SYSREG_TPIDR_EL1,
};

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
fn jit_read_sysreg_reads_kernel_thread_and_mmu_registers() {
    let mut machine = Machine::new(1);
    machine.cpus[0].sys.tpidr_el1 = 0x1111_2222_3333_4444;
    machine.cpus[0].sys.tcr_el1 = 0x5555_6666_7777_8888;

    let tpidr = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_TPIDR_EL1)
        .expect("JIT sysreg helper should read TPIDR_EL1");
    let tcr = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_TCR_EL1)
        .expect("JIT sysreg helper should read TCR_EL1");

    assert_eq!(tpidr, 0x1111_2222_3333_4444);
    assert_eq!(tcr, 0x5555_6666_7777_8888);
}

#[test]
fn jit_read_sysreg_reads_daif_from_pstate() {
    let mut machine = Machine::new(1);
    machine.cpus[0].pstate = machine.cpus[0].pstate.with_daif(PSTATE_DAIF_MASK);

    let value = jit_read_sysreg_from_machine(&mut machine, 0, SYSREG_DAIF)
        .expect("JIT sysreg helper should read DAIF");

    assert_eq!(value, PSTATE_DAIF_MASK);
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
