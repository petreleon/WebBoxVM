use super::super::protocol::*;
use super::super::resource::FORMAT_B8G8R8A8_UNORM;
use super::super::{MAX_RESOURCES, SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{create_2d, full_scanout, header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

#[test]
fn backing_detach_and_resource_unref_are_checked() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(
        &mut gpu,
        &mut mem,
        create_2d(1, FORMAT_B8G8R8A8_UNORM, 5, 5),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, attach(1, 4096), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        resource(CMD_RESOURCE_DETACH_BACKING, 1),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        resource(CMD_RESOURCE_DETACH_BACKING, 1),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        resource(CMD_RESOURCE_UNREF, 1),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        resource(CMD_RESOURCE_UNREF, 1),
        RESP_ERR_INVALID_RESOURCE_ID,
    );
    assert_eq!(gpu.allocated_resource_bytes, 0);
}

#[test]
fn resource_count_and_formats_are_bounded() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(
        &mut gpu,
        &mut mem,
        create_2d(1, 67, 1, 1),
        RESP_ERR_INVALID_PARAMETER,
    );
    for id in 1..=MAX_RESOURCES as u32 {
        assert_response(
            &mut gpu,
            &mut mem,
            create_2d(id, FORMAT_B8G8R8A8_UNORM, 1, 1),
            RESP_OK_NODATA,
        );
    }
    assert_response(
        &mut gpu,
        &mut mem,
        create_2d(10_000, FORMAT_B8G8R8A8_UNORM, 1, 1),
        RESP_ERR_OUT_OF_MEMORY,
    );
}

#[test]
fn unref_selected_resource_discards_pending_damage() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(
        &mut gpu,
        &mut mem,
        create_2d(1, FORMAT_B8G8R8A8_UNORM, SCANOUT_WIDTH, SCANOUT_HEIGHT),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, full_scanout(1), RESP_OK_NODATA);
    gpu.add_damage(
        1,
        Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        },
    );
    assert_response(
        &mut gpu,
        &mut mem,
        resource(CMD_RESOURCE_UNREF, 1),
        RESP_OK_NODATA,
    );
    assert!(gpu.take_scanout_update().is_empty());
}

fn attach(id: u32, len: u32) -> Vec<u8> {
    let mut bytes = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut bytes, id);
    push_u32(&mut bytes, 1);
    push_u64(&mut bytes, RAM_BASE);
    push_u32(&mut bytes, len);
    push_u32(&mut bytes, 0);
    bytes
}

fn resource(command: u32, id: u32) -> Vec<u8> {
    let mut bytes = header(command);
    push_u32(&mut bytes, id);
    push_u32(&mut bytes, 0);
    bytes
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: Vec<u8>, expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, &command)), expected);
}
