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
