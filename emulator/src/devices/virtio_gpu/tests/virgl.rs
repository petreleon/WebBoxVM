use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::super::{SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{full_scanout, header, response_type};
use crate::memory::PhysicalMemory;

const VIRGL_FORMAT_B8G8R8A8_UNORM: u32 = 1;
const VIRGL_TARGET_TEXTURE_2D: u32 = 2;
#[test]
fn virgl_capset_one_is_standard_sized_and_conservative() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let response = gpu.execute_command(&mut mem, &capset_info(0));
    assert_eq!(response_type(&response), RESP_OK_CAPSET_INFO);
    assert_eq!(read_u32(&response, 24), Some(VIRGL_CAPSET_ID));
    assert_eq!(read_u32(&response, 28), Some(1));
    assert_eq!(read_u32(&response, 32), Some(308));

    let response = gpu.execute_command(&mut mem, &capset(VIRGL_CAPSET_ID, 1));
    assert_eq!(response_type(&response), RESP_OK_CAPSET);
    assert_eq!(response.len(), 24 + 308);
    assert_eq!(read_u32(&response, 24), Some(1));
    assert_eq!(read_u32(&response, 28), Some(2));
    assert_eq!(read_u32(&response, 92), Some(30));
    assert_eq!(read_u32(&response, 304), Some(1));
}
#[test]
fn virgl_surface_clear_is_deferred_until_webgpu_ack_effect() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
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

    let color = [0.25, 0.5, 0.75, 1.0];
    let result = gpu.execute_queued_command(&mut mem, &submit(&surface_clear(9, color)));
    assert_eq!(response_type(&result.response), RESP_OK_NODATA);
    assert_eq!(result.deferred.map(|deferred| deferred.sequence), Some(1));
    assert_eq!(&gpu.resources[&4].pixels[..4], &[0, 0, 0, 0]);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGC1");
    assert_eq!(read_u32(&packet, 4), Some(1));
    assert_eq!(read_u32(&packet, 8), Some(1));
    assert_eq!(read_u32(&packet, 12), Some(SCANOUT_WIDTH));
    assert_eq!(read_u32(&packet, 16), Some(SCANOUT_HEIGHT));

    let effect = gpu.pending_3d[0]
        .effect
        .clone()
        .expect("VirGL clear has an ack effect");
    assert!(gpu.apply_3d_effect(effect));
    assert_eq!(&gpu.resources[&4].pixels[..4], &[191, 128, 64, 255]);
    assert_eq!(&gpu.take_scanout_update()[..4], b"WBGF");
}
#[test]
fn virgl_rejects_unsupported_streams_without_creating_a_surface() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &resource_create(4), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &context_resource(CMD_CTX_ATTACH_RESOURCE, 4),
        RESP_OK_NODATA,
    );
    let mut invalid = surface_create(9, 4);
    invalid[0] = command_header(1, 6, 5);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&invalid),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&surface_clear(9, [0.0; 4])),
        RESP_ERR_INVALID_PARAMETER,
    );
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

fn virgl_context() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 5);
    push_u32(&mut command, VIRGL_CAPSET_ID);
    command.extend_from_slice(b"virgl");
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

fn surface_clear(handle: u32, color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![command_header(62, 0, 10), 8, handle];
    words.extend(color.map(f32::to_bits));
    words.extend([0, 0, SCANOUT_WIDTH, SCANOUT_HEIGHT]);
    words
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
