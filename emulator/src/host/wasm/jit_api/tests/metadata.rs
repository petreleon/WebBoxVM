use crate::host::wasm::Emulator;

#[test]
fn jit_last_block_metadata_packs_cached_fields() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_last_block_steps = 7;
    emulator.jit_last_block_start_pc = 0x1000;
    emulator.jit_last_block_start_pa = 0x4000_1000;
    emulator.jit_last_block_exit_pc = 0x1020;
    emulator.jit_last_block_alternate_exit_pc = 0x1030;
    emulator.jit_last_block_dynamic_exit = true;
    emulator.jit_last_block_raw_hash = 0xaabb;
    emulator.jit_last_block_memory_generation = 10;
    emulator.jit_last_block_start_page_generation = 11;
    emulator.jit_last_block_end_page_generation = 12;

    assert_eq!(
        emulator.jit_last_block_metadata(),
        vec![
            7,
            0x1000,
            0x4000_1000,
            0x1020,
            0x1030,
            1,
            0xaabb,
            10,
            11,
            12
        ]
    );
}
