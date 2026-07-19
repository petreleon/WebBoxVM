use super::super::finish::finish_jit_block;
use crate::arch::arm64::CpuLifecycle;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::constants::RAM_BASE;
use crate::host::wasm::JitPendingStore;
use crate::runtime::Machine;

#[test]
fn multicore_jit_store_wakes_a_remote_wfe_but_not_wfi() {
    let mut machine = Machine::new(3);
    let store_pa = RAM_BASE + 0x100;
    machine.cpus[0].reserve_exclusive(store_pa, 8);
    machine.cpus[1].reserve_exclusive(store_pa, 8);
    machine.cpus[1].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[1].waiting_for_event = true;
    machine.cpus[2].reserve_exclusive(store_pa, 8);
    machine.cpus[2].lifecycle = CpuLifecycle::WaitingForInterrupt;
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

    assert!(machine.cpus[0].exclusive.is_none());
    assert!(!machine.cpus[0].event_register);
    assert!(machine.cpus[1].exclusive.is_none());
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[1].waiting_for_event);
    assert!(!machine.cpus[1].event_register);
    assert!(machine.cpus[2].exclusive.is_none());
    assert_eq!(machine.cpus[2].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(!machine.cpus[2].waiting_for_event);
    assert!(machine.cpus[2].event_register);
    assert_eq!(machine.bus.mem.read(store_pa, 4), Some(0x0403_0201));
    assert_eq!(machine.active_core, 1);
}
