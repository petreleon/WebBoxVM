use super::*;
use crate::arch::arm64::jit::{WasmJitCpuState, hash_raw_words};
use crate::constants::TIMER_CTL_ENABLE;
use crate::host::wasm::jit_api::prepare::prepare_jit_block;
use crate::host::wasm::jit_api::validate::code_page_generations;

#[test]
fn prepare_jit_block_copies_validated_core_state() {
    let mut machine = Machine::new(1);
    machine.cpus[0].regs.pc = RAM_BASE;
    machine.cpus[0].regs.set_x(0, 0x1234);
    machine.bus.mem.write(RAM_BASE, 4, NOP as u64);
    let hash = hash_raw_words(RAM_BASE, [NOP]);
    let memory_generation = machine.bus.mem.generation();
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, RAM_BASE, 1).expect("code page generations");
    let mut state = WasmJitCpuState::default();

    prepare_jit_block(
        &mut machine,
        &mut state,
        0,
        RAM_BASE,
        RAM_BASE,
        hash,
        memory_generation,
        start_generation,
        end_generation,
        1,
    )
    .expect("prepare valid JIT block");

    assert_eq!(state.pc, RAM_BASE);
    assert_eq!(state.x[0], 0x1234);
}

#[test]
fn prepare_jit_block_rejects_timer_boundary_before_copying_state() {
    let mut machine = Machine::new(1);
    let cpu = &mut machine.cpus[0];
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(0, 0x1234);
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 103;
    let mut state = WasmJitCpuState::default();

    let err = prepare_jit_block(
        &mut machine,
        &mut state,
        0,
        RAM_BASE,
        RAM_BASE,
        0,
        0,
        0,
        0,
        4,
    )
    .expect_err("timer preflight must reject before validation/sync");

    assert!(err.contains("timer deadline"), "{err}");
    assert_eq!(state.pc, 0);
    assert_eq!(state.x[0], 0);
}
