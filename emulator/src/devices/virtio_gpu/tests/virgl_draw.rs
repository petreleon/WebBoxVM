use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{full_scanout, header, response_type, virgl_source_over_state};
use crate::memory::PhysicalMemory;

pub(super) const BUFFER: u32 = 5;
pub(super) const TARGET: u32 = 4;
pub(super) const VERT: &str =
    "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
pub(super) const FRAG: &str =
    "FRAG\nDCL OUT[0], COLOR\nIMM[0] FLT32 {0, 1, 0, .25}\n0: MOV OUT[0], IMM[0]\n1: END\n";

#[test]
fn standard_draw_vbo_queues_a_webgpu_triangle_after_clear() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_vertices(&mut gpu);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&draw()),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());

    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGD1");
    assert_eq!(packet.len(), 104);
    assert_eq!(read_u32(&packet, 8), Some(1));
    assert_eq!(read_u32(&packet, 20), Some(3));
    assert_eq!(read_u32(&packet, 52), Some(0.25f32.to_bits()));
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    assert_eq!(&gpu.resources[&TARGET].pixels[..4], &[77, 51, 26, 255]);
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[middle..middle + 4],
        &[58, 102, 20, 255]
    );
    assert_eq!(&gpu.take_scanout_update()[..4], b"WBGF");
}
pub(super) fn prepared() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &texture_create(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &buffer_create(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(TARGET), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &context(), RESP_OK_NODATA);
    for resource in [TARGET, BUFFER] {
        assert_response(&mut gpu, &mut mem, &attach(resource), RESP_OK_NODATA);
    }
    (gpu, mem)
}
fn texture_create() -> Vec<u8> {
    create(TARGET, 2, 1, 2, 1024, 768)
}
fn buffer_create() -> Vec<u8> {
    create(BUFFER, 0, 31, 1 << 4, 48, 1)
}

fn create(id: u32, target: u32, format: u32, bind: u32, width: u32, height: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [id, target, format, bind, width, height, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}
fn context() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    for value in [5, VIRGL_CAPSET_ID] {
        push_u32(&mut command, value);
    }
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    command
}

fn attach(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [resource, 0] {
        push_u32(&mut command, value);
    }
    command
}

pub(super) fn submit(words: &[u32]) -> Vec<u8> {
    let mut command = header(CMD_SUBMIT_3D);
    for value in [(words.len() * 4) as u32, 0] {
        push_u32(&mut command, value);
    }
    for word in words {
        push_u32(&mut command, *word);
    }
    command
}

pub(super) fn surface_create(handle: u32, resource: u32) -> Vec<u32> {
    vec![word(1, 7, 5), handle, resource, 1, 0, 0]
}
pub(super) fn framebuffer(handle: u32) -> Vec<u32> {
    vec![word(5, 0, 3), 1, 0, handle]
}
pub(super) fn vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 5), 9, 0, 0, 0, 31],
        vec![word(2, 5, 1), 9],
        vec![word(6, 0, 3), 16, 0, BUFFER],
    ]
    .concat()
}

pub(super) fn shader_create(handle: u32, kind: u32, source: &str) -> Vec<u32> {
    let mut bytes = source.as_bytes().to_vec();
    bytes.push(0);
    let mut words = vec![
        word(1, 4, (5 + bytes.len().div_ceil(4)) as u16),
        handle,
        kind,
        bytes.len() as u32,
        8,
        0,
    ];
    for chunk in bytes.chunks(4) {
        let mut value = [0; 4];
        value[..chunk.len()].copy_from_slice(chunk);
        words.push(u32::from_le_bytes(value));
    }
    words
}

pub(super) fn shader_bind(handle: u32, kind: u32) -> Vec<u32> {
    vec![word(29, 0, 2), handle, kind]
}

pub(super) fn clear(color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(7, 0, 8), 4];
    words.extend(color.map(f32::to_bits));
    words.extend([0; 3]);
    words
}

pub(super) fn draw() -> Vec<u32> {
    vec![word(8, 0, 12), 0, 3, 4, 0, 1, 0, 0, 0, 0, 0, u32::MAX, 0]
}
pub(super) fn word(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

pub(super) fn upload_vertices(gpu: &mut VirtioGpu) {
    let positions = [
        0.0, 0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = positions.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources
        .get_mut(&BUFFER)
        .unwrap()
        .pixels
        .copy_from_slice(&bytes);
}

pub(super) fn assert_response(
    gpu: &mut VirtioGpu,
    mem: &mut PhysicalMemory,
    command: &[u8],
    expected: u32,
) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
