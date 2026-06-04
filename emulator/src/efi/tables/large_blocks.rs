use super::*;

pub(super) fn install_specialized_trampolines(bus: &mut SystemBus, con_out_struct: u64) {
    install_allocate_pool(bus);
    install_memory_map(bus);
    install_protocol_trampolines(bus);
    install_locate_protocol(bus);
    install_console_output(bus, con_out_struct);
    install_memory_helpers(bus);
    install_page_allocators(bus);
}

fn install_allocate_pool(bus: &mut SystemBus) {
    let addr = EFI_LARGE_CODE_ADDR;
    let bump = bump_allocator_trampoline(EFI_POOL_HEAD_PTR);
    write_trampoline(bus, addr, &bump);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_ALLOCATE_POOL_OFFSET, addr);
    write64(bus, EFI_POOL_HEAD_PTR, EFI_POOL_BASE);
}

fn install_memory_map(bus: &mut SystemBus) {
    let addr = EFI_LARGE_CODE_ADDR + LARGE_CODE_BLOCK_SIZE;
    let code = trampolines::build_get_memory_map_trampoline();
    assert!(code.len() * 4 <= LARGE_CODE_BLOCK_SIZE as usize);
    write_trampoline(bus, addr, &code);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_GET_MEMORY_MAP_OFFSET, addr);
}

fn install_protocol_trampolines(bus: &mut SystemBus) {
    let lip_addr = loaded_image_protocol_addr();
    let code = trampolines::build_handle_protocol_trampoline(lip_addr);
    let handle_addr = EFI_LARGE_CODE_ADDR + 2 * LARGE_CODE_BLOCK_SIZE;
    assert!(code.len() * 4 <= LARGE_CODE_BLOCK_SIZE as usize);
    write_trampoline(bus, handle_addr, &code);
    write64(
        bus,
        EFI_BOOT_SERVICES_ADDR + BS_HANDLE_PROTOCOL_OFFSET,
        handle_addr,
    );

    let open_addr = EFI_LARGE_CODE_ADDR + 3 * LARGE_CODE_BLOCK_SIZE;
    write_trampoline(bus, open_addr, &code);
    write64(
        bus,
        EFI_BOOT_SERVICES_ADDR + BS_OPEN_PROTOCOL_OFFSET,
        open_addr,
    );
}

fn install_locate_protocol(bus: &mut SystemBus) {
    let code = trampolines::build_locate_protocol_trampoline();
    let addr = EFI_LARGE_CODE_ADDR + 4 * LARGE_CODE_BLOCK_SIZE;
    write_trampoline(bus, addr, &code);
    write64(
        bus,
        EFI_BOOT_SERVICES_ADDR + BS_LOCATE_PROTOCOL_OFFSET,
        addr,
    );
}

fn install_console_output(bus: &mut SystemBus, con_out_struct: u64) {
    let reset_addr = EFI_LARGE_CODE_ADDR + 5 * LARGE_CODE_BLOCK_SIZE;
    super::super::encode::write_success_trampoline(bus, reset_addr, EFI_SUCCESS);
    write64(bus, con_out_struct + 0x00, reset_addr);

    let output_addr = EFI_LARGE_CODE_ADDR + 6 * LARGE_CODE_BLOCK_SIZE;
    let output_insts = [
        0x79400022, 0x350000a2, 0xD2A12003, 0x380000a2, 0x91000821, 0x17FFFFFB, 0xD2800000,
        INSTR_RET,
    ];
    write_trampoline(bus, output_addr, &output_insts);
    write64(bus, con_out_struct + 0x08, output_addr);
}

fn install_memory_helpers(bus: &mut SystemBus) {
    let copy_addr = EFI_LARGE_CODE_ADDR + 7 * LARGE_CODE_BLOCK_SIZE;
    let copy_insts = [
        0xB40000A2, 0x38401823, 0x38001803, 0xF1000442, 0x17FFFFFC, 0xD2800000, INSTR_RET,
    ];
    write_trampoline(bus, copy_addr, &copy_insts);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_COPY_MEM_OFFSET, copy_addr);

    let set_addr = EFI_LARGE_CODE_ADDR + 8 * LARGE_CODE_BLOCK_SIZE;
    let set_insts = [
        0xB4000081, 0x38001802, 0xF1000421, 0x17FFFFFD, 0xD2800000, INSTR_RET,
    ];
    write_trampoline(bus, set_addr, &set_insts);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_SET_MEM_OFFSET, set_addr);
}

fn install_page_allocators(bus: &mut SystemBus) {
    let alloc_addr = EFI_LARGE_CODE_ADDR + 9 * LARGE_CODE_BLOCK_SIZE;
    let alloc_insts = trampolines::build_allocate_pages_trampoline(
        trampolines::EFI_PAGE_ALLOC_HEAD,
        PAGE_ALLOCATOR_BASE,
    );
    write_trampoline(bus, alloc_addr, &alloc_insts);
    write64(
        bus,
        EFI_BOOT_SERVICES_ADDR + BS_ALLOCATE_PAGES_OFFSET,
        alloc_addr,
    );
    write64(bus, trampolines::EFI_PAGE_ALLOC_HEAD, PAGE_ALLOCATOR_BASE);

    let free_addr = EFI_LARGE_CODE_ADDR + 10 * LARGE_CODE_BLOCK_SIZE;
    super::super::encode::write_success_trampoline(bus, free_addr, EFI_SUCCESS);
    write64(
        bus,
        EFI_BOOT_SERVICES_ADDR + BS_FREE_PAGES_OFFSET,
        free_addr,
    );
}
