use super::super::finish::finish_jit_block;
use crate::arch::arm64::CpuLifecycle;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::constants::RAM_BASE;
use crate::host::wasm::JitPendingStore;
use crate::runtime::Machine;

#[test]
fn multicore_jit_store_still_invalidates_a_remote_exclusive_reservation() {
    let mut machine = Machine::new(2);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    let store_pa = RAM_BASE + 0x100;
    machine.cpus[1].reserve_exclusive(store_pa, 8);
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[0]);
    state.pc = RAM_BASE + 4;
    let stores = [JitPendingStore::new(store_pa, &[1, 2, 3, 4])];

    finish_jit_block(
        &state,
        &mut machine,
        0,
        1,
        RAM_BASE + 4,
        &stores,
        None,
        None,
        false,
    )
    .expect("multicore JIT state and store should commit");

    assert!(machine.cpus[1].exclusive.is_none());
    assert_eq!(machine.bus.mem.read(store_pa, 4), Some(0x0403_0201));
    assert_eq!(machine.active_core, 1);
}
