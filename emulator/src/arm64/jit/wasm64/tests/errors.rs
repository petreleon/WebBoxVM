use super::*;

#[test]
fn unsupported_opcode_error_names_opcode() {
    let err = WasmJitError::UnsupportedFirstOpcode {
        op: Opcode::Str,
        raw: 0xf900_03e0,
    };

    assert_eq!(
        err.to_string(),
        "first opcode is not wasm-jittable: Str (10) raw=0xf90003e0"
    );
}

#[test]
fn unsupported_first_opcode_is_rejected() {
    let block = block(vec![instr(Opcode::Ldr, 0, 1, 0, 0, true)]);
    let err = Wasm64Compiler::compile(&block).unwrap_err();

    assert_eq!(
        err.to_string(),
        "first opcode is not wasm-jittable: Ldr (7) raw=0x00000000"
    );
}
