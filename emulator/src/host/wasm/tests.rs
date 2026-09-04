use super::*;

#[test]
fn machine_address_is_stable_when_emulator_moves() {
    let mut emulator = Emulator::new(Some(1));
    let machine = std::ptr::from_ref(emulator.machine.as_ref());
    let token = emulator.parallel_begin_kernel(1).unwrap();
    let moved = Box::new(emulator);
    assert_eq!(machine, std::ptr::from_ref(moved.machine.as_ref()));
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn boot_machine_address_is_stable_when_emulator_moves() {
    let mut emulator = Emulator::new(Some(1));
    emulator.boot = Some(Box::new(BootContext {
        machine: Machine::new(1),
        dtb_addr: 0,
    }));
    let machine = std::ptr::from_ref(&emulator.boot.as_ref().unwrap().machine);
    let token = emulator.parallel_begin_kernel(1).unwrap();
    let moved = Box::new(emulator);
    assert_eq!(
        machine,
        std::ptr::from_ref(&moved.boot.as_ref().unwrap().machine)
    );
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn gpu_scanout_export_is_empty_without_flushed_damage() {
    let mut emulator = Emulator::new(Some(1));
    assert!(emulator.gpu_scanout_update().is_empty());
    emulator.boot = Some(Box::new(BootContext {
        machine: Machine::new(1),
        dtb_addr: 0,
    }));
    assert!(emulator.gpu_scanout_update().is_empty());
}

#[test]
fn gpu_3d_export_and_completion_are_empty_without_a_submission() {
    let mut emulator = Emulator::new(Some(1));
    assert_eq!(emulator.gpu_reset_generation(), 0);
    assert!(emulator.gpu_3d_update().is_empty());
    assert!(!emulator.gpu_3d_complete(1, true));
    assert!(!emulator.gpu_3d_complete_readback(1, 1, vec![0; 4]));
    emulator.boot = Some(Box::new(BootContext {
        machine: Machine::new(1),
        dtb_addr: 0,
    }));
    assert_eq!(emulator.gpu_reset_generation(), 0);
    assert!(emulator.gpu_3d_update().is_empty());
    assert!(!emulator.gpu_3d_complete(1, false));
    assert!(!emulator.gpu_3d_complete_readback(1, 2, vec![0; 4]));
}
