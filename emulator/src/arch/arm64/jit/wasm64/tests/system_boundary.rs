use super::*;
use crate::constants::{SYSREG_CNTPCT_EL0, SYSREG_CNTVCT_EL0, SYSREG_DAIF};

#[test]
fn daif_changes_stay_in_the_interpreter() {
    let msr_daif = crate::arch::arm64::decode(0xd51b_4233).expect("decode observed MSR DAIF");
    assert_eq!(
        (msr_daif.op, msr_daif.imm as u16),
        (Opcode::Msr, SYSREG_DAIF)
    );

    for boundary in [
        instr(Opcode::DaifSet, 0, 0, 0, 2, true),
        instr(Opcode::DaifClr, 0, 0, 0, 2, true),
        msr_daif,
    ] {
        let err = Wasm64Compiler::compile(&block(vec![boundary]))
            .expect_err("a DAIF-changing first instruction must not enter JIT");
        assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
    }
}

#[test]
fn daif_change_ends_the_compiled_prefix_before_following_work() {
    let module = Wasm64Compiler::compile(&block(vec![
        instr(Opcode::Nop, 0, 0, 0, 0, true),
        instr(Opcode::DaifClr, 0, 0, 0, 2, true),
        instr(Opcode::Nop, 0, 0, 0, 0, true),
    ]))
    .expect("compile prefix before DAIFClr");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}

#[test]
fn architectural_counter_reads_stay_in_the_interpreter() {
    for sysreg in [SYSREG_CNTPCT_EL0, SYSREG_CNTVCT_EL0] {
        let counter_read = Instr {
            imm: sysreg as u64,
            ..instr(Opcode::Mrs, 0, 0, 0, 0, true)
        };
        let err = Wasm64Compiler::compile(&block(vec![counter_read]))
            .expect_err("counter read needs an instruction-precise cycle value");
        assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));

        let module = Wasm64Compiler::compile(&block(vec![
            instr(Opcode::Nop, 0, 0, 0, 0, true),
            counter_read,
            instr(Opcode::Nop, 0, 0, 0, 0, true),
        ]))
        .expect("compile prefix before counter read");
        assert_eq!(module.guest_instr_count, 1);
        assert_eq!(module.exit_pc, 0x1004);
    }
}
