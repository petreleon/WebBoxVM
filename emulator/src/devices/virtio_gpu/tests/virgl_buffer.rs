use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const BUFFER_ID: u32 = 1;
const BUFFER_BYTES: u32 = 12;
const SECOND_BACKING: u64 = RAM_BASE + 0x100;

#[test]
fn vertex_buffer_transfer_uses_standard_byte_boxes_and_scatter_backing() {
    let (mut gpu, mut mem) = prepared_buffer();
    let upload = [1, 2, 3, 4, 5, 6, 7, 8];
    mem.write_bytes(RAM_BASE + 2, &upload[..4]).unwrap();
    mem.write_bytes(SECOND_BACKING, &upload[4..]).unwrap();

    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_TO_HOST_3D, 3, 0, 8, 2),
        RESP_OK_NODATA,
    );
    assert_eq!(&gpu.resources[&BUFFER_ID].pixels[3..11], upload);
    gpu.resources.get_mut(&BUFFER_ID).unwrap().pixels[3..11]
        .copy_from_slice(&[8, 7, 6, 5, 4, 3, 2, 1]);

    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_FROM_HOST_3D, 3, 0, 8, 2),
        RESP_OK_NODATA,
    );
    assert_eq!(read(&mem, RAM_BASE + 2, 4), [8, 7, 6, 5]);
    assert_eq!(read(&mem, SECOND_BACKING, 4), [4, 3, 2, 1]);
}

#[test]
fn vertex_buffers_reject_nonbuffer_layouts_and_texture_stream_commands() {
    let (mut gpu, mut mem) = prepared_buffer();
    let before = gpu.resources[&BUFFER_ID].pixels.clone();
    for command in [
        transfer(CMD_TRANSFER_TO_HOST_3D, 11, 0, 2, 0),
        transfer(CMD_TRANSFER_TO_HOST_3D, 0, 1, 1, 0),
        transfer(CMD_TRANSFER_FROM_HOST_3D, 8, 0, 4, 10),
    ] {
        assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
    }
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&[command_header(1, 7, 5), 9, BUFFER_ID, 64, 0, 0]),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(gpu.resources[&BUFFER_ID].pixels, before);

    let mut invalid = buffer_create(2, BUFFER_BYTES);
    invalid[44..48].copy_from_slice(&2u32.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &invalid, RESP_ERR_INVALID_PARAMETER);
}

fn prepared_buffer() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(
        &mut gpu,
        &mut mem,
        &buffer_create(BUFFER_ID, BUFFER_BYTES),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, &attach_backing(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    push_u32(&mut attach, BUFFER_ID);
    push_u32(&mut attach, 0);
    assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    (gpu, mem)
}

fn buffer_create(id: u32, width: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [id, 0, 64, 1 << 4, width, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn attach_backing() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut command, BUFFER_ID);
    push_u32(&mut command, 2);
    for (addr, len) in [(RAM_BASE, 6), (SECOND_BACKING, 6)] {
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

fn transfer(kind: u32, x: u32, y: u32, width: u32, offset: u64) -> Vec<u8> {
    let mut command = header(kind);
    for value in [x, y, 0, width, 1, 1] {
        push_u32(&mut command, value);
    }
    push_u64(&mut command, offset);
    for value in [BUFFER_ID, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn submit(words: &[u32]) -> Vec<u8> {
    let mut command = header(CMD_SUBMIT_3D);
    push_u32(&mut command, (words.len() * 4) as u32);
    push_u32(&mut command, 0);
    for word in words {
        push_u32(&mut command, *word);
    }
    command
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn read(mem: &PhysicalMemory, addr: u64, len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    mem.read_bytes(addr, &mut bytes).unwrap();
    bytes
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
