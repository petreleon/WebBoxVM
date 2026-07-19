use super::*;
use crate::arch::arm64::decode;

#[test]
fn compiles_noop_hint_aliases() {
    let block = block(vec![
        instr(Opcode::Autiasp, 0, 0, 0, 0, true),
        instr(Opcode::BtiC, 0, 0, 0, 0, true),
        instr(Opcode::Prfm, 0, 0, 0, 0, true),
        instr(Opcode::Yield, 0, 0, 0, 0, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile no-op aliases");

    assert_eq!(module.guest_instr_count, 4);
    assert_eq!(module.exit_pc, 0x1010);
}

#[test]
fn compiles_observed_dmb_as_noop() {
    let instr = decode(0xd503_39bf).expect("decode dmb ishld");
    assert_eq!(instr.op, Opcode::Dmb);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile dmb");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}

#[test]
fn event_hints_end_the_compiled_prefix() {
    for event in [Opcode::Wfe, Opcode::Sev, Opcode::Sevl] {
        let module = Wasm64Compiler::compile(&block(vec![
            instr(Opcode::Nop, 0, 0, 0, 0, true),
            instr(event, 0, 0, 0, 0, true),
            instr(Opcode::Nop, 0, 0, 0, 0, true),
        ]))
        .expect("compile prefix before event instruction");
        assert_eq!(module.guest_instr_count, 1, "{event:?}");
        assert_eq!(module.exit_pc, 0x1004, "{event:?}");
    }
}

#[test]
fn event_hint_at_block_start_requires_interpreter() {
    for event in [Opcode::Wfe, Opcode::Sev, Opcode::Sevl] {
        let error = Wasm64Compiler::compile(&block(vec![instr(event, 0, 0, 0, 0, true)]))
            .expect_err("event instruction must not be emitted as a Wasm no-op");
        assert!(
            error.to_string().contains("not wasm-jittable"),
            "{event:?}: {error}"
        );
    }
}
