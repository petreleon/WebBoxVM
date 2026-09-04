use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::memory::PhysicalMemory;

const DEPTH: u32 = 7;
const DEPTH_SURFACE: u32 = 10;
const COLOR_SURFACE: u32 = 9;
const DSA: u32 = 15;
const CONSTANT_FRAG: &str = "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn standard_less_depth_draws_batch_in_one_deferred_submission() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem, DEPTH); configure(&mut gpu, &mut mem); upload_depth_vertices(&mut gpu);
    let mut command = clear();
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5]));
    let mut far = draw(); far[1] = 3; command.extend(far);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1");
    assert_eq!([4, 12, 16, 20, 24, 44].map(|at| read_u32(&packet, at)), [Some(2), Some(1024), Some(768), Some(2), Some(0), Some(1.0f32.to_bits())]);
    assert_eq!(packet.len(), 264);
    assert_eq!([48, 156].map(|at| read_u32(&packet, at)), [Some(3), Some(3)]);
    let effect = gpu.pending_3d[0].effect.clone().expect("depth batch effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[0, 0, 128, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[middle..middle + 4].try_into().unwrap()), 0.25);
}

#[test]
fn depth_batch_rejects_mixed_depth_attachments_before_queueing() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem, DEPTH); add_depth(&mut gpu, &mut mem, 8); configure(&mut gpu, &mut mem); upload_depth_vertices(&mut gpu);
    let mut surface = surface_create(16, 8); surface[3] = 18;
    assert_response(&mut gpu, &mut mem, &submit(&surface), RESP_OK_NODATA);
    let mut command = clear(); command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend([word(5, 0, 3), 1, 16, COLOR_SURFACE]); command.extend(constants([0.0, 1.0, 0.0, 0.5]));
    let mut far = draw(); far[1] = 3; command.extend(far);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.take_3d_update().is_empty());
}

#[test]
fn standard_equal_depth_draws_batch_in_one_deferred_submission() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem, DEPTH); configure(&mut gpu, &mut mem); upload_equal_depth_vertices(&mut gpu);
    let state = [word(3, 0, 1), DSA, word(1, 0, 5), DSA, 11, 0, 0, 0, word(2, 0, 1), DSA];
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    let mut command = clear(); command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 0.0, 1.0, 0.5]));
    let mut second = draw(); second[1] = 3; command.extend(second);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 12, 16, 20, 24, 44].map(|at| read_u32(&packet, at)), [Some(3), Some(1024), Some(768), Some(2), Some(2), Some(1.0f32.to_bits())]);
    assert_eq!(packet.len(), 264);
    let effect = gpu.pending_3d[0].effect.clone().expect("equal depth batch effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[128, 0, 64, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[middle..middle + 4].try_into().unwrap()), 1.0);
}

#[test]
fn standard_mixed_depth_compares_batch_in_one_deferred_submission() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem, DEPTH); configure(&mut gpu, &mut mem); upload_depth_vertices(&mut gpu);
    let mut command = clear(); command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend([word(1, 0, 5), DSA + 1, 19, 0, 0, 0, word(2, 0, 1), DSA + 1]);
    command.extend(constants([0.0, 1.0, 0.0, 0.5]));
    let mut far = draw(); far[1] = 3; command.extend(far);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 12, 16, 20, 24, 44, 52, 160, 164].map(|at| read_u32(&packet, at)), [Some(4), Some(1024), Some(768), Some(2), Some(0), Some(1.0f32.to_bits()), Some(1), Some(3), Some(4)]);
    assert_eq!(packet.len(), 272);
    let effect = gpu.pending_3d[0].effect.clone().expect("mixed depth batch effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[0, 128, 64, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[middle..middle + 4].try_into().unwrap()), 0.75);
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut PhysicalMemory) {
    let mut state = surface_create(COLOR_SURFACE, TARGET);
    state.extend(surface_create(DEPTH_SURFACE, DEPTH).into_iter().enumerate().map(|(index, word)| if index == 3 { 18 } else { word }));
    state.extend([word(5, 0, 3), 1, DEPTH_SURFACE, COLOR_SURFACE]);
    state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, CONSTANT_FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    state.extend([word(1, 0, 5), DSA, 7, 0, 0, 0, word(2, 0, 1), DSA]);
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn clear() -> Vec<u32> {
    let mut words = vec![word(7, 0, 8), 5];
    words.extend([0.0, 0.0, 0.0, 1.0].map(f32::to_bits)); words.extend([1.0f32.to_bits(), 0, 0]);
    words
}

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 6), 1, 0]; words.extend(color.map(f32::to_bits)); words
}

fn add_depth(gpu: &mut super::super::VirtioGpu, mem: &mut PhysicalMemory, resource: u32) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [resource, 2, 18, 1, 1024, 768, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [resource, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn upload_depth_vertices(gpu: &mut super::super::VirtioGpu) {
    let mut bytes = Vec::new();
    for z in [-0.5, 0.5] {
        for [x, y] in [[0.0, 0.75], [-0.75, -0.75], [0.75, -0.75]] {
            bytes.extend([x, y, z, 1.0].into_iter().flat_map(f32::to_le_bytes));
        }
    }
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

fn upload_equal_depth_vertices(gpu: &mut super::super::VirtioGpu) {
    let mut bytes = Vec::new();
    for _ in 0..2 {
        for [x, y] in [[0.0, 0.75], [-0.75, -0.75], [0.75, -0.75]] {
            bytes.extend([x, y, 1.0, 1.0].into_iter().flat_map(f32::to_le_bytes));
        }
    }
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
