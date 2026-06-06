use super::*;
use crate::constants::{SYSREG_ICC_IAR1_EL1, SYSREG_SP_EL0};

#[test]
fn compiles_mrs_with_sysreg_helper_import() {
    let block = block(vec![Instr {
        imm: SYSREG_SP_EL0 as u64,
        ..instr(Opcode::Mrs, 4, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile MRS");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(module
        .bytes
        .windows(b"jitReadSysReg".len())
        .any(|w| w == b"jitReadSysReg"));
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
