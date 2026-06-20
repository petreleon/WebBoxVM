use super::*;
use crate::constants::{
    SYSREG_CNTVCT_EL0, SYSREG_DAIF, SYSREG_ICC_IAR1_EL1, SYSREG_SP_EL0, SYSREG_TCR_EL1,
    SYSREG_TPIDR_EL0, SYSREG_TPIDR_EL1,
};

#[test]
fn compiles_mrs_with_sysreg_helper_import() {
    let block = block(vec![Instr {
        imm: SYSREG_SP_EL0 as u64,
        ..instr(Opcode::Mrs, 4, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile MRS");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(
        module
            .bytes
            .windows(b"jitReadSysReg".len())
            .any(|w| w == b"jitReadSysReg")
    );
}

#[test]
fn compiles_observed_mrs_tpidr_el0() {
    let instr = crate::arch::arm64::decode(0xd53b_d042).expect("decode mrs x2, tpidr_el0");
    assert_eq!(instr.op, Opcode::Mrs);
    assert_eq!((instr.rd, instr.imm as u16), (2, SYSREG_TPIDR_EL0));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS TPIDR_EL0");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
}

#[test]
fn compiles_observed_mrs_cntvct_el0() {
    let instr = crate::arch::arm64::decode(0xd53b_e040).expect("decode mrs x0, cntvct_el0");
    assert_eq!(instr.op, Opcode::Mrs);
    assert_eq!((instr.rd, instr.imm as u16), (0, SYSREG_CNTVCT_EL0));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS CNTVCT_EL0");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
}

#[test]
fn compiles_observed_mrs_kernel_sysregs() {
    let cases = [
        (0xd538_d082, SYSREG_TPIDR_EL1),
        (0xd538_2040, SYSREG_TCR_EL1),
        (0xd53b_4233, SYSREG_DAIF),
    ];
    for (raw, sysreg) in cases {
        let instr = crate::arch::arm64::decode(raw).expect("decode observed MRS");
        assert_eq!(instr.op, Opcode::Mrs);
        assert_eq!(instr.imm as u16, sysreg);

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS");

        assert_eq!(module.guest_instr_count, 1);
        assert!(module.bytes.contains(&opcodes::OP_CALL));
    }
}

#[test]
fn rejects_side_effectful_mrs_interrupt_acknowledge() {
    let block = block(vec![Instr {
        imm: SYSREG_ICC_IAR1_EL1 as u64,
        ..instr(Opcode::Mrs, 0, 0, 0, 0, true)
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject side-effectful MRS");

    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
