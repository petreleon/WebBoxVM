use super::*;

#[test]
fn unsupported_opcode_error_names_opcode() {
    let err = WasmJitError::UnsupportedFirstOpcode(Opcode::Str);

    assert_eq!(
        err.to_string(),
        "first opcode is not wasm-jittable: Str (10)"
    );
}
