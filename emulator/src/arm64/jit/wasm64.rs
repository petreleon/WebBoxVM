//! ARM64-to-Wasm64 block compiler.
//!
//! The browser JIT path cannot rely on native executable memory. Instead it
//! emits small WebAssembly modules that import the existing Memory64 instance
//! and operate on a fixed CPU-state buffer:
//!
//! ```text
//! export run(state_ptr: i64) -> i64
//! ```
//!
//! This backend is intentionally conservative. It only compiles straight-line,
//! register-only instructions whose semantics are independent of memory, MMIO,
//! exceptions, timers, and system registers. Unsupported instructions are hard
//! fallback boundaries.

use super::block::Block;
use crate::arm64::{Armv8Cpu, Instr, Opcode, ProcessorState};
use crate::constants::{PAGE_OFFSET_MASK, SP_REGISTER_INDEX, ZERO_REGISTER_INDEX};
use core::fmt;

pub const JIT_STATE_X_OFFSET: u64 = 0;
pub const JIT_STATE_SP_OFFSET: u64 = 31 * 8;
pub const JIT_STATE_PC_OFFSET: u64 = 32 * 8;
pub const JIT_STATE_PSTATE_OFFSET: u64 = 33 * 8;
pub const JIT_STATE_SIZE: usize = JIT_STATE_PSTATE_OFFSET as usize + 8;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WasmJitCpuState {
    pub x: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
}

impl WasmJitCpuState {
    pub fn from_cpu(cpu: &Armv8Cpu) -> Self {
        let mut state = Self {
            sp: cpu.regs.sp,
            pc: cpu.regs.pc,
            pstate: cpu.pstate.to_u64(),
            ..Self::default()
        };
        for reg in 0..31 {
            state.x[reg] = cpu.regs.x(reg as u8);
        }
        state
    }

    pub fn copy_from_cpu(&mut self, cpu: &Armv8Cpu) {
        *self = Self::from_cpu(cpu);
    }

    pub fn copy_to_cpu(&self, cpu: &mut Armv8Cpu) {
        for reg in 0..31 {
            cpu.regs.set_x(reg as u8, self.x[reg]);
        }
        cpu.regs.sp = self.sp;
        cpu.regs.pc = self.pc;
        cpu.pstate = ProcessorState::from_u64(self.pstate);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmBlockModule {
    pub start_pc: u64,
    pub start_pa: u64,
    pub exit_pc: u64,
    pub guest_instr_count: usize,
    pub raw_hash: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmJitError {
    BlockDiscovery(&'static str),
    EmptyBlock,
    UnsupportedFirstOpcode(Opcode),
}

impl fmt::Display for WasmJitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockDiscovery(err) => write!(f, "block discovery failed: {err}"),
            Self::EmptyBlock => write!(f, "block has no instructions"),
            Self::UnsupportedFirstOpcode(op) => {
                write!(f, "first opcode is not wasm-jittable: {op:?}")
            }
        }
    }
}

impl std::error::Error for WasmJitError {}

pub struct Wasm64Compiler;

impl Wasm64Compiler {
    pub fn compile(block: &Block) -> Result<WasmBlockModule, WasmJitError> {
        if block.instructions.is_empty() {
            return Err(WasmJitError::EmptyBlock);
        }

        let mut body = WasmExpr::new();
        let mut compiled = 0usize;
        let mut raw_hash = hash_seed(block.start_pa);

        for (index, &(instr, raw)) in block.instructions.iter().enumerate() {
            let expected_pa = block.start_pa + index as u64 * 4;
            if block.instruction_pas.get(index).copied() != Some(expected_pa) {
                if compiled == 0 {
                    return Err(WasmJitError::BlockDiscovery(
                        "non-contiguous physical block",
                    ));
                }
                break;
            }
            if !body.emit_instr(instr, block.start_pc + index as u64 * 4) {
                if compiled == 0 {
                    return Err(WasmJitError::UnsupportedFirstOpcode(instr.op));
                }
                break;
            }
            raw_hash = hash_raw_word(raw_hash, raw);
            compiled += 1;
        }

        let exit_pc = block.start_pc + compiled as u64 * 4;
        body.emit_store_const(JIT_STATE_PC_OFFSET, exit_pc);
        body.i64_const(exit_pc);
        body.end();

        Ok(WasmBlockModule {
            start_pc: block.start_pc,
            start_pa: block.start_pa,
            exit_pc,
            guest_instr_count: compiled,
            raw_hash,
            bytes: build_module(body.into_bytes()),
        })
    }
}

pub fn hash_raw_words(start_pa: u64, raw_words: impl IntoIterator<Item = u32>) -> u64 {
    raw_words
        .into_iter()
        .fold(hash_seed(start_pa), hash_raw_word)
}

fn hash_seed(start_pa: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in start_pa.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_raw_word(mut hash: u64, raw: u32) -> u64 {
    for byte in raw.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

struct WasmExpr {
    code: Vec<u8>,
}

impl WasmExpr {
    fn new() -> Self {
        Self { code: Vec::new() }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.code
    }

    fn emit_instr(&mut self, instr: Instr, pc: u64) -> bool {
        match instr.op {
            Opcode::Nop | Opcode::NopBarrier => true,
            Opcode::Movz | Opcode::Movn => {
                self.emit_write_reg_with(instr.rd, instr.sf, |this| this.i64_const(instr.imm));
                true
            }
            Opcode::Movk => {
                let shift = (instr.cond as u64) * 16;
                let mask = !(0xffffu64 << shift);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rd, instr.sf);
                    this.i64_const(mask);
                    this.op(OP_I64_AND);
                    this.i64_const(instr.imm);
                    this.op(OP_I64_OR);
                });
                true
            }
            Opcode::MovReg => {
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rm, instr.sf);
                });
                true
            }
            Opcode::Sxtw => {
                self.emit_write_reg_with(instr.rd, true, |this| {
                    this.emit_read_reg(instr.rn, false);
                    this.op(OP_I32_WRAP_I64);
                    this.op(OP_I64_EXTEND_I32_S);
                });
                true
            }
            Opcode::AddImm | Opcode::SubImm => {
                let op = if instr.op == Opcode::AddImm {
                    OP_I64_ADD
                } else {
                    OP_I64_SUB
                };
                self.emit_write_reg_sp_with(instr.rd, instr.sf, |this| {
                    this.emit_read_base(instr.rn, instr.sf);
                    this.i64_const(instr.imm);
                    this.op(op);
                });
                true
            }
            Opcode::Add | Opcode::Sub if instr.cond == 0 && instr.imm == 0 => {
                let op = if instr.op == Opcode::Add {
                    OP_I64_ADD
                } else {
                    OP_I64_SUB
                };
                self.emit_write_reg_sp_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.emit_read_reg(instr.rm, instr.sf);
                    this.op(op);
                });
                true
            }
            Opcode::AndImm | Opcode::OrrImm | Opcode::EorImm => {
                let op = logical_opcode(instr.op);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.i64_const(instr.imm);
                    this.op(op);
                });
                true
            }
            Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg
                if instr.cond == 0 && instr.imm == 0 =>
            {
                let op = logical_opcode(instr.op);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.emit_read_reg(instr.rm, instr.sf);
                    this.op(op);
                });
                true
            }
            Opcode::Adr => {
                let target = (pc as i64 + instr.imm as i64) as u64;
                self.emit_write_reg_with(instr.rd, true, |this| this.i64_const(target));
                true
            }
            Opcode::Adrp => {
                let page = pc & !PAGE_OFFSET_MASK;
                let target = (page as i64 + instr.imm as i64) as u64;
                self.emit_write_reg_with(instr.rd, true, |this| this.i64_const(target));
                true
            }
            _ => false,
        }
    }

    fn emit_read_reg(&mut self, reg: u8, sf: bool) {
        if reg >= ZERO_REGISTER_INDEX {
            self.i64_const(0);
            return;
        }
        self.emit_load(reg_offset(reg));
        self.mask_32_if_needed(sf);
    }

    fn emit_read_base(&mut self, reg: u8, sf: bool) {
        let offset = if reg >= SP_REGISTER_INDEX {
            JIT_STATE_SP_OFFSET
        } else {
            reg_offset(reg)
        };
        self.emit_load(offset);
        self.mask_32_if_needed(sf);
    }

    fn emit_write_reg_with(&mut self, reg: u8, sf: bool, value: impl FnOnce(&mut Self)) {
        if reg >= ZERO_REGISTER_INDEX {
            return;
        }
        self.emit_store_with(reg_offset(reg), sf, value);
    }

    fn emit_write_reg_sp_with(&mut self, reg: u8, sf: bool, value: impl FnOnce(&mut Self)) {
        let offset = if reg >= SP_REGISTER_INDEX {
            JIT_STATE_SP_OFFSET
        } else {
            reg_offset(reg)
        };
        self.emit_store_with(offset, sf, value);
    }

    fn emit_store_const(&mut self, offset: u64, value: u64) {
        self.emit_store_with(offset, true, |this| this.i64_const(value));
    }

    fn emit_store_with(&mut self, offset: u64, sf: bool, value: impl FnOnce(&mut Self)) {
        self.emit_addr(offset);
        value(self);
        self.mask_32_if_needed(sf);
        self.op(OP_I64_STORE);
        encode_u32(&mut self.code, 3); // natural i64 alignment
        encode_u64(&mut self.code, 0);
    }

    fn emit_load(&mut self, offset: u64) {
        self.emit_addr(offset);
        self.op(OP_I64_LOAD);
        encode_u32(&mut self.code, 3); // natural i64 alignment
        encode_u64(&mut self.code, 0);
    }

    fn emit_addr(&mut self, offset: u64) {
        self.op(OP_LOCAL_GET);
        encode_u32(&mut self.code, 0);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
    }

    fn mask_32_if_needed(&mut self, sf: bool) {
        if !sf {
            self.i64_const(u32::MAX as u64);
            self.op(OP_I64_AND);
        }
    }

    fn i64_const(&mut self, value: u64) {
        self.op(OP_I64_CONST);
        encode_i64(&mut self.code, value as i64);
    }

    fn op(&mut self, opcode: u8) {
        self.code.push(opcode);
    }

    fn end(&mut self) {
        self.op(OP_END);
    }
}

fn logical_opcode(op: Opcode) -> u8 {
    match op {
        Opcode::AndImm | Opcode::AndReg => OP_I64_AND,
        Opcode::OrrImm | Opcode::OrrReg => OP_I64_OR,
        Opcode::EorImm | Opcode::EorReg => OP_I64_XOR,
        _ => unreachable!(),
    }
}

fn reg_offset(reg: u8) -> u64 {
    JIT_STATE_X_OFFSET + reg as u64 * 8
}

fn build_module(expr: Vec<u8>) -> Vec<u8> {
    let mut module = Vec::new();
    module.extend_from_slice(b"\0asm");
    module.extend_from_slice(&[1, 0, 0, 0]);
    append_section(&mut module, SECTION_TYPE, type_section());
    append_section(&mut module, SECTION_IMPORT, import_section());
    append_section(&mut module, SECTION_FUNCTION, function_section());
    append_section(&mut module, SECTION_EXPORT, export_section());
    append_section(&mut module, SECTION_CODE, code_section(expr));
    module
}

fn type_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    section.push(0x60); // function type
    encode_u32(&mut section, 1);
    section.push(TYPE_I64);
    encode_u32(&mut section, 1);
    section.push(TYPE_I64);
    section
}

fn import_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_name(&mut section, "env");
    encode_name(&mut section, "memory");
    section.push(IMPORT_MEMORY);
    encode_u32(&mut section, LIMITS_MEMORY64);
    encode_u64(&mut section, 0);
    section
}

fn function_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_u32(&mut section, 0);
    section
}

fn export_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_name(&mut section, "run");
    section.push(EXPORT_FUNC);
    encode_u32(&mut section, 0);
    section
}

fn code_section(expr: Vec<u8>) -> Vec<u8> {
    let mut body = Vec::new();
    encode_u32(&mut body, 0); // local decl count
    body.extend_from_slice(&expr);

    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_u32(&mut section, body.len() as u32);
    section.extend_from_slice(&body);
    section
}

fn append_section(module: &mut Vec<u8>, id: u8, section: Vec<u8>) {
    module.push(id);
    encode_u32(module, section.len() as u32);
    module.extend_from_slice(&section);
}

fn encode_name(dst: &mut Vec<u8>, name: &str) {
    encode_u32(dst, name.len() as u32);
    dst.extend_from_slice(name.as_bytes());
}

fn encode_u32(dst: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        dst.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_u64(dst: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        dst.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_i64(dst: &mut Vec<u8>, mut value: i64) {
    loop {
        let byte = (value as u8) & 0x7f;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        dst.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}

const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;

const TYPE_I64: u8 = 0x7e;
const IMPORT_MEMORY: u8 = 0x02;
const EXPORT_FUNC: u8 = 0x00;
const LIMITS_MEMORY64: u32 = 0x04;

const OP_END: u8 = 0x0b;
const OP_LOCAL_GET: u8 = 0x20;
const OP_I64_LOAD: u8 = 0x29;
const OP_I64_STORE: u8 = 0x37;
const OP_I64_CONST: u8 = 0x42;
const OP_I64_ADD: u8 = 0x7c;
const OP_I64_SUB: u8 = 0x7d;
const OP_I64_AND: u8 = 0x83;
const OP_I64_OR: u8 = 0x84;
const OP_I64_XOR: u8 = 0x85;
const OP_I32_WRAP_I64: u8 = 0xa7;
const OP_I64_EXTEND_I32_S: u8 = 0xac;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            module
                .bytes
                .windows(b"memory".len())
                .any(|w| w == b"memory")
        );
        assert!(module.bytes.windows(b"run".len()).any(|w| w == b"run"));
    }

    #[test]
    fn unsupported_first_opcode_is_rejected() {
        let block = block(vec![instr(Opcode::Ldr, 0, 1, 0, 0, true)]);

        assert_eq!(
            Wasm64Compiler::compile(&block),
            Err(WasmJitError::UnsupportedFirstOpcode(Opcode::Ldr))
        );
    }

    #[test]
    fn unsupported_opcode_ends_compiled_prefix() {
        let block = block(vec![
            instr(Opcode::Movz, 0, 0, 0, 5, true),
            instr(Opcode::Ldr, 1, 0, 0, 0, true),
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
}
