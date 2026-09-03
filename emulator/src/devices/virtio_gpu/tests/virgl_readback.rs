use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const RESOURCE_ID: u32 = 1;
const WIDTH: u32 = 4;
const HEIGHT: u32 = 3;
const SECOND_BACKING: u64 = RAM_BASE + 0x100;

#[test]
fn transfer_from_host_3d_writes_exact_scatter_backed_pixels() {
    let (mut gpu, mut mem) = resource_with_backing();
    gpu.resources.get_mut(&RESOURCE_ID).unwrap().pixels[4..12]
        .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    gpu.resources.get_mut(&RESOURCE_ID).unwrap().pixels[20..28]
        .copy_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
    let command = transfer(1, 0, 2, 2, 4);

    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_CONTEXT_ID);
    assert_eq!(read(&mem, RAM_BASE + 4, 8), [0; 8]);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);

    assert_eq!(read(&mem, RAM_BASE + 4, 8), [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(read(&mem, RAM_BASE + 20, 4), [9, 10, 11, 12]);
    assert_eq!(read(&mem, SECOND_BACKING, 4), [13, 14, 15, 16]);
}

#[test]
fn transfer_from_host_3d_rejects_an_incomplete_range_without_partial_writes() {
    let (mut gpu, mut mem) = resource_with_backing();
    gpu.resources.get_mut(&RESOURCE_ID).unwrap().pixels[..4].copy_from_slice(&[9; 4]);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(0, 0, 1, 1, 47),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(read(&mem, SECOND_BACKING + 23, 1), [0]);
    assert_eq!(read(&mem, RAM_BASE, 4), [0; 4]);
}

fn resource_with_backing() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &attach_backing(), RESP_OK_NODATA);
    (gpu, mem)
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
    push_u32(&mut command, 2);
    for (addr, len) in [(RAM_BASE, 24), (SECOND_BACKING, 24)] {
        push_u64(&mut command, addr);
        push_u32(&mut command, len);
        push_u32(&mut command, 0);
    }
    command
}

fn virgl_context() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 5);
    push_u32(&mut command, VIRGL_CAPSET_ID);
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    command
}

fn transfer(x: u32, y: u32, width: u32, height: u32, offset: u64) -> Vec<u8> {
    let mut command = header(CMD_TRANSFER_FROM_HOST_3D);
    for value in [x, y, 0, width, height, 1] {
        push_u32(&mut command, value);
    }
    push_u64(&mut command, offset);
    for value in [RESOURCE_ID, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn read(mem: &PhysicalMemory, addr: u64, len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    mem.read_bytes(addr, &mut bytes).unwrap();
    bytes
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
