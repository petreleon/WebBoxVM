use super::super::protocol::*;
use super::super::completion::{PendingCompletion, WritableRegion};
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::constants::RAM_BASE;

const TEXTURE_BUFFER: u32 = 8;
const UNIFORM: u32 = 9;
const TEXTURE_VERT: &str = "VERT\nDCL IN[0..2]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nDCL OUT[2], GENERIC[1]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nMOV OUT[2], IN[2]\nEND\n";
const TEXTURE_FRAG: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL IN[1], GENERIC[1], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[1], SAMP[0], 2D\nMUL OUT[0], TEMP[0], IN[0]\nEND\n";
const TEXTURE_OFFSET_VERT: &str = "VERT\nDCL IN[0..1]\nDCL CONST[0][0]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nADD OUT[0], IN[0], CONST[0][0]\nMOV OUT[1], IN[1]\nEND\n";
const TEXTURE_CONSTANT_FRAG: &str = "FRAG\nDCL CONST[0][0]\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMUL OUT[0], TEMP[0], CONST[0][0]\nEND\n";
const CONSTANT_FRAG: &str = "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn standard_mixed_material_draws_share_one_ordered_batch() {
    let (mut gpu, mut mem) = prepared();
    add_texture_buffer(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &submit(&solid_state()), RESP_OK_NODATA);
    upload_vertices(&mut gpu); upload_texture_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4).for_each(|pixel| pixel.copy_from_slice(&[128, 128, 128, 255]));
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw()); command.extend(texture_state()); command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGM1"); assert_eq!([4, 12, 16, 20, 24, 44].map(|at| read_u32(&packet, at)), [Some(1), Some(1024), Some(768), Some(2), Some(0), Some(0)]);
    assert_eq!([48, 52, 56, 164, 168, 172].map(|at| read_u32(&packet, at)), [Some(1), Some(0), Some(3), Some(5), Some(0), Some(3)]);
    assert_eq!(packet.len(), 364);
    gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels.fill(0); gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("mixed material batch effect");
    assert!(!gpu.apply_3d_readback(effect.clone(), 1, &[]));
    assert_eq!(&gpu.resources[&TARGET].pixels[..4], &[0, 0, 0, 0]);
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[32, 32, 64, 255]);
}

#[test]
fn depth_mixed_material_draws_preserve_depth_order_in_one_batch() {
    let (mut gpu, mut mem) = prepared(); add_texture_buffer(&mut gpu, &mut mem); add_uniform(&mut gpu, &mut mem); add_depth(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &submit(&depth_solid_state()), RESP_OK_NODATA);
    upload_depth_solid_vertices(&mut gpu); upload_depth_texture_vertices(&mut gpu);
    store_vertex_offset(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4).for_each(|pixel| pixel.copy_from_slice(&[128, 128, 128, 255]));
    let mut command = depth_clear(); command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw()); command.extend(texture_constant_state()); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("depth batch defers");
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion { header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head: 1 }));
    let packet = gpu.take_3d_update(); assert_eq!(&packet[..4], b"VGM1");
    assert_eq!([4, 12, 16, 20, 24, 44, 52, 100, 104, 108, 112, 168, 244, 260].map(|at| read_u32(&packet, at)), [Some(1), Some(1024), Some(768), Some(2), Some(1), Some(1.0f32.to_bits()), Some(7), Some(1.0f32.to_bits()), Some(0), Some(0), Some(0.5f32.to_bits()), Some(7), Some((-0.015625f32).to_bits()), Some(0.5f32.to_bits())]);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0); gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels.fill(0); gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let pixels = [3, 2, 1, 255].repeat(1024 * 768); assert!(gpu.complete_3d_readback(&mut mem, deferred.sequence, 2, &pixels));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[1, 2, 3, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[center..center + 4].try_into().unwrap()), 0.25);
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
}

fn add_texture_buffer(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [TEXTURE_BUFFER, 0, 31, 1 << 4, 120, 1, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE); for value in [TEXTURE_BUFFER, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn add_uniform(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [UNIFORM, 0, 64, 1 << 6, 32, 1, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE); for value in [UNIFORM, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn solid_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state()); state
}

fn depth_solid_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); let mut depth = surface_create(15, DEPTH); depth[3] = 18;
    state.extend(depth); state.extend([word(5, 0, 3), 1, 15, 9]);
    let mut fragment = shader_create(12, 1, CONSTANT_FRAG); fragment[4] = 11;
    state.extend(shader_create(11, 0, VERT)); state.extend(fragment); state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    state.extend([word(1, 0, 5), 16, 7, 0, 0, 0, word(2, 0, 1), 16]); state
}

fn texture_state() -> Vec<u32> {
    let mut vertex = shader_create(21, 0, TEXTURE_VERT); vertex[4] = 21;
    let mut fragment = shader_create(22, 1, TEXTURE_FRAG); fragment[4] = 30;
    [
        vertex, fragment, shader_bind(21, 0), shader_bind(22, 1), texture_vertex_state(),
        vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0],
        vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688],
        vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17],
    ].concat()
}

fn texture_constant_state() -> Vec<u32> {
    let mut vertex = shader_create(21, 0, TEXTURE_OFFSET_VERT); vertex[4] = 20;
    let mut fragment = shader_create(22, 1, TEXTURE_CONSTANT_FRAG); fragment[4] = 30;
    [
        vertex, fragment, shader_bind(21, 0), shader_bind(22, 1), vertex_uniform(), texture_constant_vertex_state(),
        vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0],
        vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688], constants([0.5, 0.5, 0.5, 1.0]),
        vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17],
    ].concat()
}

fn texture_vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 13), 10, 0, 0, 0, 31, 16, 0, 0, 31, 32, 0, 0, 29],
        vec![word(2, 5, 1), 10], vec![word(6, 0, 3), 40, 0, TEXTURE_BUFFER],
    ].concat()
}

fn texture_constant_vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 9), 10, 0, 0, 0, 31, 16, 0, 0, 29],
        vec![word(2, 5, 1), 10], vec![word(6, 0, 3), 24, 0, TEXTURE_BUFFER],
    ].concat()
}

fn upload_texture_vertices(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

const DEPTH: u32 = 7;

fn add_depth(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [DEPTH, 2, 18, 1, 1024, 768, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA); let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [DEPTH, 0] { push_u32(&mut attach, value); } assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn depth_clear() -> Vec<u32> {
    let mut words = vec![word(7, 0, 8), 5]; words.extend([0.25, 0.5, 0.75, 1.0].map(f32::to_bits)); words.extend([1.0f32.to_bits(), 0, 0]); words
}

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 6), 1, 0]; words.extend(color.map(f32::to_bits)); words
}

fn vertex_uniform() -> Vec<u32> {
    vec![word(27, 0, 5), 0, 0, 0, 16, UNIFORM]
}

fn store_vertex_offset(gpu: &mut super::super::VirtioGpu) {
    let bytes: Vec<u8> = [-0.015625, 0.0, 0.0, 0.0].into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels[..16].copy_from_slice(&bytes);
}

fn upload_depth_solid_vertices(gpu: &mut super::super::VirtioGpu) {
    let values = [0.0, 0.75, 0.5, 1.0, -0.75, -0.75, 0.5, 1.0, 0.75, -0.75, 0.5, 1.0];
    let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_le_bytes).collect(); gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

fn upload_depth_texture_vertices(gpu: &mut super::super::VirtioGpu) {
    let values = [0.0, 0.75, -0.5, 1.0, 0.0, 1.0, -0.75, -0.75, -0.5, 1.0, 0.0, 1.0, 0.75, -0.75, -0.5, 1.0, 0.0, 1.0];
    let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_le_bytes).collect(); gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
