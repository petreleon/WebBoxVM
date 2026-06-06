use super::*;
use crate::arm64::{Armv8Cpu, ProcessorState};

#[test]
fn jit_state_layout_matches_emitted_offsets() {
    let state = WasmJitCpuState::default();
    let base = &state as *const _ as usize;
    let simd = &state.simd as *const _ as usize;

    assert_eq!(simd - base, JIT_STATE_SIMD_OFFSET as usize);
    assert_eq!(core::mem::size_of::<WasmJitCpuState>(), JIT_STATE_SIZE);
}

#[test]
fn jit_state_roundtrips_cpu_registers_and_simd() {
    let mut cpu = Armv8Cpu::default();
    cpu.regs.set_x(0, 0x1234);
    cpu.regs.set_x(30, 0xabcd);
    cpu.regs.sp = 0x8000;
    cpu.regs.pc = 0x4000;
    cpu.pstate = ProcessorState::from_u64(0xa000_0000);
    cpu.simd[0] = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;
    cpu.simd[31] = 0xffee_ddcc_bbaa_9988_7766_5544_3322_1100;

    let mut state = WasmJitCpuState::default();
    state.copy_from_cpu(&cpu);

    let mut restored = Armv8Cpu::default();
    state.copy_to_cpu(&mut restored);

    assert_eq!(restored.regs.x(0), 0x1234);
    assert_eq!(restored.regs.x(30), 0xabcd);
    assert_eq!(restored.regs.sp, 0x8000);
    assert_eq!(restored.regs.pc, 0x4000);
    assert_eq!(restored.pstate.to_u64(), 0xa000_0000);
    assert_eq!(restored.simd[0], cpu.simd[0]);
    assert_eq!(restored.simd[31], cpu.simd[31]);
}
