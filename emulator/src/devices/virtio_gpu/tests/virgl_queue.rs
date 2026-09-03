use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::super::{SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{full_scanout, header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const DESC: u64 = RAM_BASE + 0x1000;
const AVAIL: u64 = RAM_BASE + 0x3000;
const USED: u64 = RAM_BASE + 0x4000;
const REQUEST: u64 = RAM_BASE + 0x5000;
const RESPONSE: u64 = RAM_BASE + 0x30000;

#[test]
fn queued_standard_framebuffer_clear_waits_for_ack_before_mutating_scanout() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &attach_resource(4), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&surface_create(9, 4)),
        RESP_OK_NODATA,
    );

    configure_queue(&mut gpu, &mut mem, &submit(&framebuffer_clear(9)), 24);
    assert!(!gpu.write(&mut mem, 0x050, 0, 4));
    assert_eq!(&gpu.resources[&4].pixels[..4], &[0, 0, 0, 0]);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGC1");
    let sequence = read_u32(&packet, 8).expect("VGC1 sequence");

    assert!(gpu.complete_3d(&mut mem, sequence, true));
    assert_eq!(&gpu.resources[&4].pixels[..4], &[191, 128, 64, 255]);
    assert_eq!(mem.read(USED + 2, 2), Some(1));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_OK_NODATA as u64));
    assert_eq!(&gpu.take_scanout_update()[..4], b"WBGF");
}

#[test]
fn destroyed_surface_cannot_remain_a_framebuffer_target() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &attach_resource(4), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&surface_create(9, 4)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&framebuffer_bind(9)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&surface_destroy(9)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&generic_clear()),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());
}

fn resource_create(id: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [id, 2, 1, 2, SCANOUT_WIDTH, SCANOUT_HEIGHT, 1, 1, 0, 1, 0, 0] {
        push_u32(&mut command, value);
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

fn attach_resource(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_CTX_ATTACH_RESOURCE);
    push_u32(&mut command, resource);
    push_u32(&mut command, 0);
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

fn surface_create(handle: u32, resource: u32) -> Vec<u32> {
    vec![header_word(1, 7, 5), handle, resource, 1, 0, 0]
}

fn framebuffer_clear(handle: u32) -> Vec<u32> {
    let mut words = framebuffer_bind(handle);
    words.extend(generic_clear());
    words
}

fn framebuffer_bind(handle: u32) -> Vec<u32> {
    vec![header_word(5, 0, 3), 1, 0, handle]
}

fn surface_destroy(handle: u32) -> Vec<u32> {
    vec![header_word(3, 7, 1), handle]
}

fn generic_clear() -> Vec<u32> {
    let mut words = vec![header_word(7, 0, 8), 4];
    words.extend([0.25f32, 0.5, 0.75, 1.0].map(f32::to_bits));
    words.extend([0, 0, 0]);
    words
}

fn header_word(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn configure_queue(
    gpu: &mut VirtioGpu,
    mem: &mut PhysicalMemory,
    request: &[u8],
    response_len: u32,
) {
    mem.write_bytes(REQUEST, request).expect("request memory");
    write_desc(mem, DESC, REQUEST, request.len() as u32, 1, 1);
    write_desc(mem, DESC + 16, RESPONSE, response_len, 2, 0);
    mem.write(AVAIL + 2, 2, 1).expect("avail index");
    mem.write(AVAIL + 4, 2, 0).expect("avail head");
    gpu.write(mem, 0x030, 0, 4);
    gpu.write(mem, 0x038, 8, 4);
    set_addr(gpu, mem, 0x080, DESC);
    set_addr(gpu, mem, 0x090, AVAIL);
    set_addr(gpu, mem, 0x0a0, USED);
    gpu.write(mem, 0x044, 1, 4);
}

fn set_addr(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, offset: u64, addr: u64) {
    gpu.write(mem, offset, addr as u32 as u64, 4);
    gpu.write(mem, offset + 4, addr >> 32, 4);
}

fn write_desc(mem: &mut PhysicalMemory, base: u64, addr: u64, len: u32, flags: u16, next: u16) {
    mem.write(base, 8, addr).expect("descriptor address");
    mem.write(base + 8, 4, u64::from(len))
        .expect("descriptor length");
    mem.write(base + 12, 2, u64::from(flags))
        .expect("descriptor flags");
    mem.write(base + 14, 2, u64::from(next))
        .expect("descriptor next");
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
