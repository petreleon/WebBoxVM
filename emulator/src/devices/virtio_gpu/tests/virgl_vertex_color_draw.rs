use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::memory::PhysicalMemory;

const COLOR_FRAG: &str =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL OUT[0], COLOR[0]\nMOV OUT[0], IN[0]\nEND\n";
const MODULATE_FRAG: &str = "FRAG\nDCL OUT[0], COLOR[0]\nDCL CONST[0][0]\nDCL IN[0], GENERIC[0], LINEAR\nMUL OUT[0].xyzw, CONST[0][0].xyzw, IN[0].xyzw\nEND\n";
const DEPTH: u32 = 7;
const DEPTH_SURFACE: u32 = 15;
const DSA: u32 = 16;

#[test]
fn standard_generic_vertex_colors_snapshot_and_interpolate_through_schema_seven() {
    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&color_state()), RESP_OK_NODATA);
    upload_colors(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(7));
    assert_eq!(packet.len(), 192);
    assert_eq!(&packet[40..56], &[0; 16]);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("vertex color effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[64, 64, 127, 255]);
}

#[test]
fn vertex_colors_multiply_a_fragment_constant_before_gpu_snapshot() {
    let (mut gpu, mut mem) = prepared(); assert_response(&mut gpu, &mut mem, &submit(&modulated_color_state()), RESP_OK_NODATA);
    upload_colors(&mut gpu); let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(constants([0.5, 0.5, 0.5, 1.0])); command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA); let packet = gpu.take_3d_update();
    assert_eq!([4, 72, 76, 80, 84].map(|at| read_u32(&packet, at)), [Some(7), Some(0.5f32.to_bits()), Some(0), Some(0), Some(1.0f32.to_bits())]);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0); let effect = gpu.pending_3d[0].effect.clone().expect("vertex-color constant effect"); assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize; assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[32, 32, 64, 255]);
}

#[test]
fn standard_read_only_depth_vertex_colors_preserve_depth_and_interpolation() {
    let (mut gpu, mut mem) = prepared();
    add_depth(&mut gpu, &mut mem);
    let mut state = color_state();
    let mut depth_surface = surface_create(DEPTH_SURFACE, DEPTH);
    depth_surface[3] = 18;
    state.extend(depth_surface);
    state.extend([word(5, 0, 3), 1, DEPTH_SURFACE, 9]);
    state.extend([word(1, 0, 5), DSA, 5, 0, 0, 0, word(2, 0, 1), DSA]);
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_depth_colors(&mut gpu);
    let mut command = depth_clear();
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20, 192, 196].map(|at| read_u32(&packet, at)), [Some(12), Some(3), Some(1.0f32.to_bits()), Some(5)]);
    assert_eq!(packet.len(), 200);
    let effect = gpu.pending_3d[0].effect.clone().expect("depth vertex-color effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[64, 64, 127, 255]);
    assert_eq!(f32::from_le_bytes(gpu.resources[&DEPTH].pixels[center..center + 4].try_into().unwrap()), 1.0);
}

#[test]
fn generic_color_draw_rejects_a_two_component_layout_or_nonfinite_color() {
    let (mut gpu, mut mem) = prepared();
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&color_state_with(textured_vertex_state())),
        RESP_OK_NODATA,
    );
    upload_textured_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);

    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&color_state()), RESP_OK_NODATA);
    upload_colors(&mut gpu);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[16..20].copy_from_slice(&f32::NAN.to_le_bytes());
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.take_3d_update().is_empty());
}

fn color_state() -> Vec<u32> {
    color_state_with(vertex_color_state())
}

fn modulated_color_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9)); state.extend(shader_create(11, 0, TEXTURED_VERT));
    let mut fragment = shader_create(12, 1, MODULATE_FRAG); fragment[4] = 12; state.extend(fragment);
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_color_state()); state
}

fn color_state_with(vertex_state: Vec<u32>) -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    let mut fragment = shader_create(12, 1, COLOR_FRAG);
    fragment[4] = 11;
    state.extend(fragment);
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state);
    state
}

fn vertex_color_state() -> Vec<u32> {
    [
        vec![word(1, 5, 9), 10, 0, 0, 0, 31, 16, 0, 0, 31],
        vec![word(2, 5, 1), 10],
        vec![word(6, 0, 3), 32, 0, BUFFER],
    ]
    .concat()
}

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 6), 1, 0]; words.extend(color.map(f32::to_bits)); words
}

fn upload_colors(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
        -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
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

fn upload_depth_colors(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, -0.5, 1.0, 1.0, 0.0, 0.0, 1.0,
        -0.75, -0.75, -0.5, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, -0.5, 1.0, 0.0, 0.0, 1.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
