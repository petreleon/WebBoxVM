use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{full_scanout, header, response_type};
use crate::memory::PhysicalMemory;
pub(super) const BUFFER: u32 = 5;
pub(super) const TARGET: u32 = 4;
pub(super) const TEXTURE: u32 = 6;
pub(super) const VERT: &str =
    "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
pub(super) const FRAG: &str =
    "FRAG\nDCL OUT[0], COLOR\nIMM[0] FLT32 {0, 1, 0, .25}\n0: MOV OUT[0], IMM[0]\n1: END\n";
pub(super) const TEXTURED_VERT: &str = "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
pub(super) const TEXTURED_FRAG: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMOV OUT[0], TEMP[0]\nEND\n";
pub(super) const TEXTURED_MULTIPLY_FRAG: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0..1]\nDCL SVIEW[0..1], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0..1]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nTEX TEMP[1], IN[0], SAMP[1], 2D\nMUL OUT[0], TEMP[0], TEMP[1]\nEND\n";
pub(super) fn prepared() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(
        &mut gpu,
        &mut mem,
        &create(TARGET, 2, 1, 2, 1024, 768),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &create(BUFFER, 0, 31, 1 << 4, 120, 1),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &create(TEXTURE, 2, 1, 1 << 3, 2, 2),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, &full_scanout(TARGET), RESP_OK_NODATA);
    let mut context = header(CMD_CTX_CREATE);
    for value in [5, VIRGL_CAPSET_ID] {
        push_u32(&mut context, value);
    }
    context.extend_from_slice(b"virgl");
    context.resize(96, 0);
    assert_response(&mut gpu, &mut mem, &context, RESP_OK_NODATA);
    for resource in [TARGET, BUFFER, TEXTURE] {
        let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
        for value in [resource, 0] {
            push_u32(&mut attach, value);
        }
        assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    }
    (gpu, mem)
}

fn create(id: u32, target: u32, format: u32, bind: u32, width: u32, height: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [id, target, format, bind, width, height, 1, 1, 0, 0, 0, 0] {
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
    vec![word(1, 8, 5), handle, resource, 1, 0, 0]
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

pub(super) fn textured_vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 9), 10, 0, 0, 0, 31, 16, 0, 0, 29],
        vec![word(2, 5, 1), 10],
        vec![word(6, 0, 3), 24, 0, BUFFER],
    ]
    .concat()
}

pub(super) fn shader_create(handle: u32, kind: u32, source: &str) -> Vec<u32> {
    let tokens = match source {
        VERT => 11,
        FRAG => 14,
        TEXTURED_VERT => 17,
        TEXTURED_FRAG => 25,
        TEXTURED_MULTIPLY_FRAG => 31,
        _ => 8,
    };
    let mut bytes = source.as_bytes().to_vec();
    bytes.push(0);
    let mut words = vec![
        word(1, 4, (5 + bytes.len().div_ceil(4)) as u16),
        handle,
        kind,
        bytes.len() as u32,
        tokens,
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
        .get_mut(..bytes.len())
        .unwrap()
        .copy_from_slice(&bytes);
}
pub(super) fn upload_textured_vertices(gpu: &mut VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0,
        0.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources
        .get_mut(&BUFFER)
        .unwrap()
        .pixels
        .get_mut(..bytes.len())
        .unwrap()
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
