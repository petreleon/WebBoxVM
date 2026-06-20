//! ARM64→ARM64 JIT: verbatim ALU/move/logical ops at native speed.
//! Memory/branch/system ops return to dispatcher for interpreter fallback.

use super::block::Block;
use crate::arch::arm64::Armv8Cpu;
use crate::platform::virt::SystemBus;
use std::collections::HashMap;

mod ops;
mod verbatim;

use ops::*;
use verbatim::can_emit_verbatim;

pub struct NativeBlock {
    code: Vec<u8>,
    pub guest_instr_count: usize,
    pub exit_pc: u64,
}

impl NativeBlock {
    pub unsafe fn execute(&self, cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
        let ram_ptr = bus.mem.ram_data();
        type JitFn = extern "C" fn(u64, u64, u64);
        let jit: JitFn = unsafe { std::mem::transmute(self.code.as_ptr()) };
        jit(cpu as *mut _ as u64, bus as *mut _ as u64, ram_ptr as u64);
    }
}

pub struct A64Compiler {
    blocks: HashMap<u64, NativeBlock>,
}

impl A64Compiler {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }
    pub fn get(&self, pa: u64) -> Option<&NativeBlock> {
        self.blocks.get(&pa)
    }
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn compile(
        &mut self,
        block: &Block,
        _cpu: &Armv8Cpu,
        _bus: &SystemBus,
    ) -> Result<(), &'static str> {
        let mut code: Vec<u8> = Vec::new();
        let mut compiled_count = 0usize;

        emit_prologue(&mut code);
        emit_mov(&mut code, 19, 0);
        emit_mov(&mut code, 20, 1);
        emit_mov(&mut code, 21, 2);

        for i in (0..28).step_by(2) {
            let off = i * 8;
            let ldp = 0xA9400000
                | i as u32
                | ((i as u32 + 1) << 10)
                | (19u32 << 5)
                | encode_ldp_offset(off);
            emit_a64(&mut code, ldp);
        }

        for &(_, raw) in &block.instructions {
            if can_emit_verbatim(block.instructions[compiled_count].0.op) {
                emit_a64(&mut code, raw);
                compiled_count += 1;
            } else {
                break;
            }
        }

        for i in (0..28).step_by(2) {
            let off = i * 8;
            let stp = 0xA9000000
                | i as u32
                | ((i as u32 + 1) << 10)
                | (19u32 << 5)
                | encode_ldp_offset(off);
            emit_a64(&mut code, stp);
        }
        emit_epilogue(&mut code);

        let native = NativeBlock {
            code,
            guest_instr_count: compiled_count,
            exit_pc: block.start_pc + (compiled_count as u64) * 4,
        };
        self.blocks.insert(block.start_pa, native);
        Ok(())
    }
}
