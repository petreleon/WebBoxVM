use super::*;
use crate::arm64::{Armv8Cpu, Instr, Opcode, ProcessorState};
mod add_carry;
mod add_sub;
mod bit_count;
mod bitfield;
mod cmp_flags;
mod cond_select;
mod conditional_compare;
mod errors;
mod extract;
mod hints;
mod logical_flags;
mod memory_load;
mod memory_pair;
mod memory_store;
mod memory_zero;
mod multiply;
mod rev;
mod system_reg;
mod terminal_branch;
mod variable_shift;
fn block(instructions: Vec<Instr>) -> Block {
    let instruction_pas = (0..instructions.len())
        .map(|idx| 0x4000_1000 + idx as u64 * 4)
        .collect();
    Block {
        start_pc: 0x1000,
        start_pa: 0x4000_1000,
        instruction_pas,
        instructions: instructions.into_iter().map(|instr| (instr, 0)).collect(),
    }
}

fn instr(op: Opcode, rd: u8, rn: u8, rm: u8, imm: u64, sf: bool) -> Instr {
    Instr {
        op,
        rd,
        rn,
        rm,
        imm,
        sf,
        cond: 0,
        size: 0,
    }
}

fn instr_cond(op: Opcode, cond: u8, imm: u64, sf: bool) -> Instr {
    Instr {
        cond,
        imm,
        ..instr(op, 2, 0, 1, 0, sf)
    }
}

#[test]
fn jit_state_roundtrips_cpu_registers() {
    let mut cpu = Armv8Cpu::default();
    cpu.regs.set_x(0, 0x1234);
    cpu.regs.set_x(30, 0xabcd);
    cpu.regs.sp = 0x8000;
    cpu.regs.pc = 0x4000;
    cpu.pstate = ProcessorState::from_u64(0xa000_0000);

    let mut state = WasmJitCpuState::default();
    state.copy_from_cpu(&cpu);

    let mut restored = Armv8Cpu::default();
    state.copy_to_cpu(&mut restored);

    assert_eq!(restored.regs.x(0), 0x1234);
    assert_eq!(restored.regs.x(30), 0xabcd);
    assert_eq!(restored.regs.sp, 0x8000);
    assert_eq!(restored.regs.pc, 0x4000);
    assert_eq!(restored.pstate.to_u64(), 0xa000_0000);
}

#[test]
fn compiles_register_only_prefix_to_memory64_module() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
        instr(Opcode::EorImm, 2, 1, 0, 3, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile wasm64 block");

    assert_eq!(module.start_pc, 0x1000);
    assert_eq!(module.start_pa, 0x4000_1000);
    assert_eq!(module.exit_pc, 0x100c);
    assert_eq!(module.guest_instr_count, 3);
    assert_eq!(module.raw_hash, hash_raw_words(0x4000_1000, [0, 0, 0]));
    assert_eq!(&module.bytes[..8], b"\0asm\x01\0\0\0");
    assert!(module.bytes.windows(b"env".len()).any(|w| w == b"env"));
    assert!(module
        .bytes
        .windows(b"memory".len())
        .any(|w| w == b"memory"));
    assert!(module.bytes.windows(b"run".len()).any(|w| w == b"run"));
}

#[test]
fn unsupported_opcode_ends_compiled_prefix() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr(Opcode::Str, 1, 0, 0, 0, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile prefix");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}

#[test]
fn compiles_shifted_register_prefix() {
    let block = block(vec![
        instr_cond(Opcode::Add, 0, 4, true),
        instr_cond(Opcode::Sub, 1, 5, true),
        instr_cond(Opcode::AndReg, 2, 6, false),
        instr_cond(Opcode::OrrReg, 4, 0, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile shifted prefix");

    assert_eq!(module.guest_instr_count, 4);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_S));
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
}

#[test]
fn unsupported_32_bit_rotate_right_ends_compiled_prefix() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr_cond(Opcode::EorReg, 3, 5, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile prefix");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}

#[test]
fn non_contiguous_physical_address_ends_compiled_prefix() {
    let mut block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);
    block.instruction_pas[1] += 0x1000;

    let module = Wasm64Compiler::compile(&block).expect("compile prefix");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}
