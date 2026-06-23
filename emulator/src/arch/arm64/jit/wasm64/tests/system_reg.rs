use super::*;
use crate::constants::{
    SYSREG_CNTVCT_EL0, SYSREG_CURRENTEL, SYSREG_DAIF, SYSREG_DCZID_EL0, SYSREG_ICC_IAR1_EL1,
    SYSREG_SP_EL0, SYSREG_SPSR_EL1, SYSREG_TCR_EL1, SYSREG_TPIDR_EL0, SYSREG_TPIDR_EL1,
    SYSREG_TPIDRRO_EL0,
};

#[test]
fn compiles_mrs_with_sysreg_helper_import() {
    let block = block(vec![Instr {
        imm: SYSREG_SP_EL0 as u64,
        ..instr(Opcode::Mrs, 4, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile MRS");

    assert_eq!(module.guest_instr_count, 1);
    assert!(run_body_contains_call(&module.bytes));
    assert!(
        module
            .bytes
            .windows(b"jitReadSysReg".len())
            .any(|w| w == b"jitReadSysReg")
    );
}

#[test]
fn compiles_observed_mrs_tpidr_el0() {
    let cases = [
        (0xd53b_d042, 2, SYSREG_TPIDR_EL0),
        (0xd53b_00e3, 3, SYSREG_DCZID_EL0),
    ];
    for (raw, rd, sysreg) in cases {
        let instr = crate::arch::arm64::decode(raw).expect("decode observed thread-pointer MRS");
        assert_eq!(instr.op, Opcode::Mrs);
        assert_eq!((instr.rd, instr.imm as u16), (rd, sysreg));

        let module =
            Wasm64Compiler::compile(&block(vec![instr])).expect("compile thread-pointer MRS");

        assert_eq!(module.guest_instr_count, 1);
        assert!(run_body_contains_call(&module.bytes));
    }
}

#[test]
fn compiles_mrs_tpidrro_el0() {
    let block = block(vec![Instr {
        imm: SYSREG_TPIDRRO_EL0 as u64,
        ..instr(Opcode::Mrs, 5, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile MRS TPIDRRO_EL0");

    assert_eq!(module.guest_instr_count, 1);
    assert!(run_body_contains_call(&module.bytes));
}

#[test]
fn compiles_observed_mrs_cntvct_el0() {
    let instr = crate::arch::arm64::decode(0xd53b_e040).expect("decode mrs x0, cntvct_el0");
    assert_eq!(instr.op, Opcode::Mrs);
    assert_eq!((instr.rd, instr.imm as u16), (0, SYSREG_CNTVCT_EL0));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS CNTVCT_EL0");

    assert_eq!(module.guest_instr_count, 1);
    assert!(run_body_contains_call(&module.bytes));
}

#[test]
fn compiles_helper_backed_observed_mrs_kernel_sysregs() {
    let cases = [
        (0xd538_d082, SYSREG_TPIDR_EL1),
        (0xd538_2040, SYSREG_TCR_EL1),
        (0xd538_4017, SYSREG_SPSR_EL1),
    ];
    for (raw, sysreg) in cases {
        let instr = crate::arch::arm64::decode(raw).expect("decode observed MRS");
        assert_eq!(instr.op, Opcode::Mrs);
        assert_eq!(instr.imm as u16, sysreg);

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS");

        assert_eq!(module.guest_instr_count, 1);
        assert!(run_body_contains_call(&module.bytes));
    }
}

#[test]
fn inlines_pstate_backed_mrs_kernel_sysregs() {
    let cases = [(0xd538_4253, SYSREG_CURRENTEL), (0xd53b_4233, SYSREG_DAIF)];
    for (raw, sysreg) in cases {
        let instr = crate::arch::arm64::decode(raw).expect("decode observed MRS");
        assert_eq!(instr.op, Opcode::Mrs);
        assert_eq!(instr.imm as u16, sysreg);

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MRS");
        let body = run_body(&module.bytes);

        assert_eq!(module.guest_instr_count, 1);
        assert!(!body.contains(&opcodes::OP_CALL));
        assert!(body.contains(&opcodes::OP_I64_AND));
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

fn run_body_contains_call(module: &[u8]) -> bool {
    run_body(module).contains(&opcodes::OP_CALL)
}

fn run_body(module: &[u8]) -> &[u8] {
    let mut offset = 8;
    while offset < module.len() {
        let id = module[offset];
        offset += 1;
        let (section_len, payload_at) = read_var_u32(module, offset);
        offset = payload_at;
        let section_end = offset + section_len as usize;
        if id == opcodes::SECTION_CODE {
            let (func_count, body_len_at) = read_var_u32(module, offset);
            assert_eq!(func_count, 1);
            let (body_len, body_at) = read_var_u32(module, body_len_at);
            return &module[body_at..body_at + body_len as usize];
        }
        offset = section_end;
    }
    panic!("compiled module has no code section");
}

fn read_var_u32(bytes: &[u8], mut offset: usize) -> (u32, usize) {
    let mut value = 0;
    let mut shift = 0;
    loop {
        let byte = bytes[offset];
        offset += 1;
        value |= ((byte & 0x7f) as u32) << shift;
        if byte & 0x80 == 0 {
            return (value, offset);
        }
        shift += 7;
    }
}
