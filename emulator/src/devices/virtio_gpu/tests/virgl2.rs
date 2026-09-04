use super::super::protocol::*;
use super::super::three_d::VIRGL2_CAPSET_ID;
use super::super::{SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const VIRGL_FORMAT_B8G8R8A8_UNORM: u32 = 1;
const VIRGL_TARGET_TEXTURE_2D: u32 = 2;

#[test]
fn virgl2_capset_has_the_current_growable_layout_without_unbacked_features() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let response = gpu.execute_command(&mut mem, &capset_info(1));
    assert_eq!(response_type(&response), RESP_OK_CAPSET_INFO);
    assert_eq!(read_u32(&response, 24), Some(VIRGL2_CAPSET_ID));
    assert_eq!(read_u32(&response, 28), Some(2));
    assert_eq!(read_u32(&response, 32), Some(1376));

    let response = gpu.execute_command(&mut mem, &capset(VIRGL2_CAPSET_ID, 2));
    assert_eq!(response_type(&response), RESP_OK_CAPSET);
    assert_eq!(response.len(), 24 + 1376);
    assert_eq!(read_u32(&response, 24), Some(2));
    assert_eq!(read_u32(&response, 28), Some(2));
    assert_eq!(read_u32(&response, 92), Some(30));
    assert_eq!(read_u32(&response, 312), Some(112));
    assert!(response[24 + 308..].iter().all(|byte| *byte == 0));
}

#[test]
fn virgl2_context_routes_the_bounded_resident_clear_path() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &context_resource(CMD_CTX_ATTACH_RESOURCE, 4),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&surface_create(9, 4)),
        RESP_OK_NODATA,
    );
    let result = gpu.execute_queued_command(&mut mem, &submit(&surface_clear(9)));
    assert_eq!(response_type(&result.response), RESP_OK_NODATA);
    assert_eq!(result.deferred.map(|submit| submit.sequence), Some(1));
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGC1");
    assert_eq!([4, 36].map(|offset| read_u32(&packet, offset)), [Some(2), Some(0)]);
    assert_eq!(read_u32(&packet, 12), Some(SCANOUT_WIDTH));
    assert_eq!(read_u32(&packet, 16), Some(SCANOUT_HEIGHT));
}

#[test]
fn virgl2_context_accepts_the_bounded_standard_texture_transfer() {
    let mut gpu = VirtioGpu::new(); let mut mem = PhysicalMemory::new();
    for command in [resource_create(4), attach_backing(4), context_create(), context_resource(CMD_CTX_ATTACH_RESOURCE, 4)] {
        assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);
    }
    mem.write_bytes(RAM_BASE, &[1, 2, 3, 4]).expect("guest texture source");
    assert_response(&mut gpu, &mut mem, &transfer_to_host(4), RESP_OK_NODATA);
    assert_eq!(&gpu.resources[&4].pixels[..4], &[1, 2, 3, 4]);
}

fn capset_info(index: u32) -> Vec<u8> {
    let mut command = header(CMD_GET_CAPSET_INFO);
    push_u32(&mut command, index);
    push_u32(&mut command, 0);
    command
}

fn capset(id: u32, version: u32) -> Vec<u8> {
    let mut command = header(CMD_GET_CAPSET);
    push_u32(&mut command, id);
    push_u32(&mut command, version);
    command
}

fn context_create() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 6);
    push_u32(&mut command, VIRGL2_CAPSET_ID);
    command.extend_from_slice(b"virgl2");
    command.resize(96, 0);
    command
}

fn resource_create(id: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [
        id,
        VIRGL_TARGET_TEXTURE_2D,
        VIRGL_FORMAT_B8G8R8A8_UNORM,
        2,
        SCANOUT_WIDTH,
        SCANOUT_HEIGHT,
        1,
        1,
        0,
        1,
        0,
        0,
    ] {
        push_u32(&mut command, value);
    }
    command
}

fn full_scanout(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_SET_SCANOUT);
    for value in [0, 0, SCANOUT_WIDTH, SCANOUT_HEIGHT, 0, resource] {
        push_u32(&mut command, value);
    }
    command
}

fn attach_backing(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_ATTACH_BACKING); push_u32(&mut command, resource); push_u32(&mut command, 1);
    push_u64(&mut command, RAM_BASE); push_u32(&mut command, SCANOUT_WIDTH * SCANOUT_HEIGHT * 4); push_u32(&mut command, 0); command
}

fn transfer_to_host(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_TRANSFER_TO_HOST_3D);
    for value in [0, 0, 0, 1, 1, 1] { push_u32(&mut command, value); }
    push_u64(&mut command, 0); for value in [resource, 0, 0, 0] { push_u32(&mut command, value); } command
}

fn context_resource(kind: u32, resource: u32) -> Vec<u8> {
    let mut command = header(kind);
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
    vec![
        command_header(1, 8, 5),
        handle,
        resource,
        VIRGL_FORMAT_B8G8R8A8_UNORM,
        0,
        0,
    ]
}

fn surface_clear(handle: u32) -> Vec<u32> {
    let mut words = vec![command_header(62, 0, 10), 8, handle];
    words.extend([0.25f32, 0.5, 0.75, 1.0].map(f32::to_bits));
    words.extend([0, 0, SCANOUT_WIDTH, SCANOUT_HEIGHT]);
    words
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
