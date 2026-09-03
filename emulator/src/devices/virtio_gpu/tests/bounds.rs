use super::super::protocol::*;
use super::super::resource::FORMAT_B8G8R8A8_UNORM;
use super::super::{MAX_TOTAL_RESOURCE_BYTES, SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{append_rect, create_2d, full_scanout, header, response_type};
use crate::memory::PhysicalMemory;

#[test]
fn zero_width_and_height_are_rejected() {
    let mut gpu = VirtioGpu::new();
    let mem = PhysicalMemory::new();
    for (id, width, height) in [(1, 0, 1), (2, 1, 0)] {
        let response =
            gpu.execute_command(&mem, &create_2d(id, FORMAT_B8G8R8A8_UNORM, width, height));
        assert_eq!(response_type(&response), RESP_ERR_INVALID_PARAMETER);
    }
    assert!(gpu.resources.is_empty());
}

#[test]
fn aggregate_limit_rejects_a_resource_without_retaining_an_allocation() {
    let mut gpu = VirtioGpu::new();
    let mem = PhysicalMemory::new();
    gpu.allocated_resource_bytes = MAX_TOTAL_RESOURCE_BYTES;
    let command = create_2d(1, FORMAT_B8G8R8A8_UNORM, 4096, 4096);
    let response = gpu.execute_command(&mem, &command);
    assert_eq!(response_type(&response), RESP_ERR_OUT_OF_MEMORY);
    assert!(gpu.resources.is_empty());
    assert_eq!(gpu.allocated_resource_bytes, MAX_TOTAL_RESOURCE_BYTES);
}

#[test]
fn flush_of_unselected_resource_succeeds_without_visible_damage() {
    let mut gpu = VirtioGpu::new();
    let mem = PhysicalMemory::new();
    for id in [1, 2] {
        assert_ok(
            &mut gpu,
            &mem,
            &create_2d(id, FORMAT_B8G8R8A8_UNORM, SCANOUT_WIDTH, SCANOUT_HEIGHT),
        );
    }
    assert_ok(&mut gpu, &mem, &full_scanout(1));
    let mut flush = header(CMD_RESOURCE_FLUSH);
    append_rect(
        &mut flush,
        Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        },
    );
    push_u32(&mut flush, 2);
    push_u32(&mut flush, 0);
    assert_ok(&mut gpu, &mem, &flush);
    assert!(gpu.take_scanout_update().is_empty());
}

fn assert_ok(gpu: &mut VirtioGpu, mem: &PhysicalMemory, command: &[u8]) {
    assert_eq!(
        response_type(&gpu.execute_command(mem, command)),
        RESP_OK_NODATA
    );
}
