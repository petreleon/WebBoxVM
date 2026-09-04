use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::memory::PhysicalMemory;

const DEPTH: u32 = 7;
const DEPTH_SURFACE: u32 = 15;
const DSA: u32 = 16;

#[test]
fn standard_depth_textured_draw_snapshots_the_sampler_and_writes_depth() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &submit(&depth_texture_state()), RESP_OK_NODATA);
    upload_depth_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.copy_from_slice(&[
        10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
    ]);
    let mut command = depth_clear();
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20, 196, 200].map(|at| read_u32(&packet, at)), [Some(13), Some(3), Some(1.0f32.to_bits()), Some(7)]);
    assert_eq!(packet.len(), 204);
    assert_eq!(&packet[180..196], &[10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255]);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("depth textured effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[10, 20, 30, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[center..center + 4].try_into().unwrap()), 0.25);
}

fn depth_texture_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    let mut depth_surface = surface_create(DEPTH_SURFACE, DEPTH);
    depth_surface[3] = 18;
    state.extend(depth_surface);
    state.extend([word(5, 0, 3), 1, DEPTH_SURFACE, 9]);
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend([word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0]);
    state.extend([word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688]);
    state.extend([word(10, 0, 3), 1, 0, 18, word(18, 0, 3), 1, 0, 17]);
    state.extend([word(1, 0, 5), DSA, 7, 0, 0, 0, word(2, 0, 1), DSA]);
    state
}

fn add_depth(gpu: &mut super::super::VirtioGpu, mem: &mut PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [DEPTH, 2, 18, 1, 1024, 768, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [DEPTH, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn depth_clear() -> Vec<u32> {
    let mut words = vec![word(7, 0, 8), 5];
    words.extend([0.1, 0.2, 0.3, 1.0].map(f32::to_bits));
    words.extend([1.0f32.to_bits(), 0, 0]);
    words
}

fn upload_depth_vertices(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, -0.5, 1.0, 0.0, 1.0, -0.75, -0.75, -0.5, 1.0, 0.0, 1.0,
        0.75, -0.75, -0.5, 1.0, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
