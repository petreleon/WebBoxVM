use super::super::protocol::*;
use super::virgl_draw::{
    FRAG, TARGET, VERT, assert_response, draw, prepared, shader_bind, shader_create, submit,
    surface_create, vertex_state, word,
};
use super::{header, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::memory::PhysicalMemory;

const DEPTH: u32 = 7;
const DEPTH_SURFACE: u32 = 10;
const COLOR_SURFACE: u32 = 9;
const DSA: u32 = 15;

#[test]
fn standard_less_depth_state_blocks_the_later_far_triangle() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem);
    let mut state = surface_create(COLOR_SURFACE, TARGET);
    state.extend(surface_create(DEPTH_SURFACE, DEPTH).into_iter().enumerate().map(|(i, word)| {
        if i == 3 { 18 } else { word }
    }));
    state.extend([word(5, 0, 3), 1, DEPTH_SURFACE, COLOR_SURFACE]);
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    state.extend([word(1, 0, 5), DSA, 7, 0, 0, 0, word(2, 0, 1), DSA]);
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_overlapping_vertices(&mut gpu);

    let mut invalid = clear(false);
    invalid.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&invalid), RESP_ERR_INVALID_PARAMETER);
    let mut command = clear(true);
    let mut call = draw();
    call[2] = 6;
    command.extend(call);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20, 192].map(|offset| read_u32(&packet, offset)), [Some(9), Some(6), Some(1.0f32.to_bits())]);
    assert_eq!(packet.len(), 196);
    let effect = gpu.pending_3d[0].effect.clone().expect("depth draw effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[58, 102, 20, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[middle..middle + 4].try_into().unwrap()), 0.25);
}

#[test]
fn standard_equal_depth_state_accepts_the_far_plane_triangle() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem);
    let mut state = surface_create(COLOR_SURFACE, TARGET);
    state.extend(surface_create(DEPTH_SURFACE, DEPTH).into_iter().enumerate().map(|(i, word)| {
        if i == 3 { 18 } else { word }
    }));
    state.extend([word(5, 0, 3), 1, DEPTH_SURFACE, COLOR_SURFACE]);
    state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state()); state.extend([word(1, 0, 5), DSA, 11, 0, 0, 0, word(2, 0, 1), DSA]);
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_equal_vertices(&mut gpu);
    let mut command = clear(true); command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20, 144, 148].map(|offset| read_u32(&packet, offset)), [Some(10), Some(3), Some(1.0f32.to_bits()), Some(2)]);
    assert_eq!(packet.len(), 152);
    let effect = gpu.pending_3d[0].effect.clone().expect("equal depth draw effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[58, 102, 20, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[middle..middle + 4].try_into().unwrap()), 1.0);
}

#[test]
fn depth_state_rejects_noncanonical_dsa_payloads() {
    let (mut gpu, mut mem) = prepared();
    for state in [vec![word(1, 0, 5), DSA, 2, 0, 0, 0], vec![word(1, 0, 5), DSA, 35, 0, 0, 0], vec![word(3, 0, 1), 0]] {
        assert_response(&mut gpu, &mut mem, &submit(&state), RESP_ERR_INVALID_PARAMETER);
    }
}

fn add_depth(gpu: &mut super::super::VirtioGpu, mem: &mut PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [DEPTH, 2, 18, 1, 1024, 768, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [DEPTH, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn clear(depth: bool) -> Vec<u32> {
    let mut words = vec![word(7, 0, 8), if depth { 5 } else { 4 }];
    words.extend([0.1f32, 0.2, 0.3, 1.0].map(f32::to_bits));
    words.extend([if depth { 1.0f32.to_bits() } else { 0 }, 0, 0]);
    words
}

fn upload_overlapping_vertices(gpu: &mut super::super::VirtioGpu) {
    let mut vertices = Vec::new();
    for z in [-0.5, 0.5] {
        for [x, y] in [[0.0, 0.75], [-0.75, -0.75], [0.75, -0.75]] {
            vertices.extend([x, y, z, 1.0].into_iter().flat_map(f32::to_le_bytes));
        }
    }
    gpu.resources.get_mut(&5).unwrap().pixels[..vertices.len()].copy_from_slice(&vertices);
}

fn upload_equal_vertices(gpu: &mut super::super::VirtioGpu) {
    let mut vertices = Vec::new();
    for [x, y] in [[0.0, 0.75], [-0.75, -0.75], [0.75, -0.75]] {
        vertices.extend([x, y, 1.0, 1.0].into_iter().flat_map(f32::to_le_bytes));
    }
    gpu.resources.get_mut(&5).unwrap().pixels[..vertices.len()].copy_from_slice(&vertices);
}
