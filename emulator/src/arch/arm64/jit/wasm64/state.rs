use crate::arch::arm64::{Armv8Cpu, ProcessorState};

pub const JIT_STATE_X_OFFSET: u64 = 0;
pub const JIT_STATE_SP_OFFSET: u64 = 31 * 8;
pub const JIT_STATE_PC_OFFSET: u64 = 32 * 8;
pub const JIT_STATE_PSTATE_OFFSET: u64 = 33 * 8;
pub const JIT_STATE_SP_EL0_OFFSET: u64 = JIT_STATE_PSTATE_OFFSET + 8;
pub const JIT_STATE_SIMD_OFFSET: u64 = ((JIT_STATE_SP_EL0_OFFSET + 8 + 15) / 16) * 16;
pub const JIT_STATE_SIZE: usize = JIT_STATE_SIMD_OFFSET as usize + 32 * 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WasmJitCpuState {
    pub x: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
    pub sp_el0: u64,
    pub simd: [u128; 32],
}

impl WasmJitCpuState {
    pub fn from_cpu(cpu: &Armv8Cpu) -> Self {
        let mut state = Self {
            sp: cpu.regs.sp,
            pc: cpu.regs.pc,
            pstate: cpu.pstate.to_u64(),
            sp_el0: cpu.sys.sp_el0,
            ..Self::default()
        };
        for reg in 0..31 {
            state.x[reg] = cpu.regs.x(reg as u8);
        }
        state.simd = cpu.simd;
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
        cpu.sys.sp_el0 = self.sp_el0;
        cpu.simd = self.simd;
    }
}
