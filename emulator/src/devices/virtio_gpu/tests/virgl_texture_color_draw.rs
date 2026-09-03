use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const VERT: &str = "VERT\nDCL IN[0..2]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nDCL OUT[2], GENERIC[1]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nMOV OUT[2], IN[2]\nEND\n";
const FRAG: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL IN[1], GENERIC[1], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[1], SAMP[0], 2D\nMUL OUT[0], TEMP[0], IN[0]\nEND\n";

#[test]
fn standard_texture_color_draw_snapshots_generic_rgba_and_sampler_data() {
    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA);
    upload(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4)
        .for_each(|pixel| pixel.copy_from_slice(&[128, 128, 128, 255]));
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(8));
    assert_eq!(packet.len(), 244);
    assert_eq!(&packet[40..56], &[0; 16]);
    assert_eq!(read_u32(&packet, 216), Some(0x1092));
    assert_eq!([220, 224].map(|offset| read_u32(&packet, offset)), [Some(2); 2]);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("texture-color effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[32, 32, 64, 255]);
}

#[test]
fn texture_color_draw_rejects_wrong_layout_and_nonfinite_uv() {
    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&state_with(textured_vertex_state())), RESP_OK_NODATA);
    upload(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);

    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA);
    upload(&mut gpu);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[36..40].copy_from_slice(&f32::NAN.to_le_bytes());
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.take_3d_update().is_empty());
}

fn state() -> Vec<u32> {
    state_with(texture_color_vertex_state())
}

fn state_with(vertex_state: Vec<u32>) -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    let mut vertex = shader_create(11, 0, VERT);
    vertex[4] = 21;
    let mut fragment = shader_create(12, 1, FRAG);
    fragment[4] = 30;
    state.extend(vertex);
    state.extend(fragment);
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state);
    state.extend(vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0]);
    state.extend(vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688]);
    state.extend(vec![word(10, 0, 3), 1, 0, 18]);
    state.extend(vec![word(18, 0, 3), 1, 0, 17]);
    state
}

fn texture_color_vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 13), 10, 0, 0, 0, 31, 16, 0, 0, 31, 32, 0, 0, 29],
        vec![word(2, 5, 1), 10], vec![word(6, 0, 3), 40, 0, BUFFER],
    ].concat()
}

fn upload(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
