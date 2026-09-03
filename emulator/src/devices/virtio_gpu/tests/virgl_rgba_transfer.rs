use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const RESOURCE: u32 = 9;
const RGBA: [u8; 16] = [
    30, 20, 10, 255, 60, 50, 40, 255, 90, 80, 70, 255, 120, 110, 100, 255,
];
const BGRA: [u8; 16] = [
    10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
];

#[test]
fn virgl_rgba_transfer_roundtrips_through_canonical_bgra_storage() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    for command in [resource_create(), context_create(), attach_backing()] {
        assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);
    }
    mem.write_bytes(RAM_BASE, &RGBA)
        .expect("guest source bytes");
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_TO_HOST_3D),
        RESP_OK_NODATA,
    );
    assert_eq!(gpu.resources[&RESOURCE].pixels, BGRA);
    mem.write_bytes(RAM_BASE, &[0; 16])
        .expect("clear guest destination");
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_FROM_HOST_3D),
        RESP_OK_NODATA,
    );
    let mut readback = [0; 16];
    mem.read_bytes(RAM_BASE, &mut readback)
        .expect("guest readback bytes");
    assert_eq!(readback, RGBA);
}

fn resource_create() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [RESOURCE, 2, 67, 1 << 3, 2, 2, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn context_create() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    for value in [5, VIRGL_CAPSET_ID] {
        push_u32(&mut command, value);
    }
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    command
}

fn attach_backing() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_ATTACH_BACKING);
    for value in [RESOURCE, 1] {
        push_u32(&mut command, value);
    }
    push_u64(&mut command, RAM_BASE);
    for value in [16, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn transfer(command_type: u32) -> Vec<u8> {
    let mut command = header(command_type);
    for value in [0, 0, 0, 2, 2, 1] {
        push_u32(&mut command, value);
    }
    push_u64(&mut command, 0);
    for value in [RESOURCE, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
