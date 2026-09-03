use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const RESOURCE_ID: u32 = 1;
const WIDTH: u32 = 4;
const HEIGHT: u32 = 3;

#[test]
fn transfer_to_host_3d_requires_a_virgl_context_then_uploads() {
    let (mut gpu, mut mem) = resource_with_backing();
    let offset = ((WIDTH + 1) * 4) as u64;
    mem.write_bytes(RAM_BASE + offset, &[1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();
    let command = transfer(1, 1, 0, 2, offset, 0, 0, 0);
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_CONTEXT_ID);

    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);
    assert_eq!(
        &gpu.resources[&RESOURCE_ID].pixels[20..28],
        &[1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert!(gpu.pending_damage.is_none());
}

#[test]
fn transfer_to_host_3d_rejects_nonclassic_layouts_without_mutation() {
    let (mut gpu, mut mem) = prepared_gpu();
    let before = gpu.resources[&RESOURCE_ID].pixels.clone();
    let nonzero_z = transfer(0, 0, 1, 1, 0, 0, 0, 0);
    let mut deep = transfer(0, 0, 0, 1, 0, 0, 0, 0);
    deep[44..48].copy_from_slice(&2u32.to_le_bytes());
    for command in [
        nonzero_z,
        deep,
        transfer(0, 0, 0, 1, 0, 1, 0, 0),
        transfer(0, 0, 0, 1, 0, 0, 4, 0),
        transfer(0, 0, 0, 1, 0, 0, 0, 16),
        header(CMD_TRANSFER_TO_HOST_3D),
    ] {
        assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
    }
    assert_eq!(gpu.resources[&RESOURCE_ID].pixels, before);
}

#[test]
fn transfer_to_host_3d_rejects_a_resource_without_backing() {
    let (mut gpu, mut mem) = resource_without_backing();
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    let before = gpu.resources[&RESOURCE_ID].pixels.clone();
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(0, 0, 0, 1, 0, 0, 0, 0),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(gpu.resources[&RESOURCE_ID].pixels, before);
}

fn prepared_gpu() -> (VirtioGpu, PhysicalMemory) {
    let (mut gpu, mut mem) = resource_with_backing();
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    (gpu, mem)
}

fn resource_with_backing() -> (VirtioGpu, PhysicalMemory) {
    let (mut gpu, mut mem) = resource_without_backing();
    assert_response(&mut gpu, &mut mem, &attach_backing(), RESP_OK_NODATA);
    (gpu, mem)
}

fn resource_without_backing() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(), RESP_OK_NODATA);
    (gpu, mem)
}

fn virgl_context() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 5);
    push_u32(&mut command, VIRGL_CAPSET_ID);
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    command
}

fn resource_create() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [RESOURCE_ID, 2, 1, 2, WIDTH, HEIGHT, 1, 1, 0, 1, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn attach_backing() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut command, RESOURCE_ID);
    push_u32(&mut command, 1);
    push_u64(&mut command, RAM_BASE);
    push_u32(&mut command, WIDTH * HEIGHT * 4);
    push_u32(&mut command, 0);
    command
}

fn transfer(
    x: u32,
    y: u32,
    z: u32,
    width: u32,
    offset: u64,
    level: u32,
    stride: u32,
    layer: u32,
) -> Vec<u8> {
    let mut command = header(CMD_TRANSFER_TO_HOST_3D);
    for value in [x, y, z, width, 1, 1] {
        push_u32(&mut command, value);
    }
    push_u64(&mut command, offset);
    for value in [RESOURCE_ID, level, stride, layer] {
        push_u32(&mut command, value);
    }
    command
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
