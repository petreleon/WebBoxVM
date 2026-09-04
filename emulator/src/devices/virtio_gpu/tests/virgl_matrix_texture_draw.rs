use super::super::protocol::*;
use super::{
    virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state,
};

const MATRIX_VERT: &str = "VERT\nDCL IN[0], POSITION\nDCL IN[1], GENERIC[0]\nDCL CONST[0..3]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[1], IN[1]\nDP4 OUT[0].w, IN[0], CONST[3]\nDP4 OUT[0].x, IN[0], CONST[0]\nDP4 OUT[0].z, IN[0], CONST[2]\nDP4 OUT[0].y, IN[0], CONST[1]\nEND\n";
const MATRIX: [f32; 16] = [
    0.5, 0.0, 0.0, 0.25,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[test]
fn matrix_vertex_preserves_one_generic_texture_varying() {
    let (mut gpu, mut mem) = prepared_nonresident();
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4).for_each(|pixel| pixel.copy_from_slice(&[10, 20, 30, 255]));
    assert_response(&mut gpu, &mut mem, &submit(&matrix(MATRIX)), RESP_OK_NODATA);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 56, 60, 64, 68].map(|at| read_u32(&packet, at)), [Some(3), Some(0.25f32.to_bits()), Some(0.375f32.to_bits()), Some(0), Some(1.0f32.to_bits())]);
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0); gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("matrix texture effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    let transformed = ((400 * 1024 + 540) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[77, 51, 26, 255]);
    assert_eq!(&gpu.resources[&TARGET].pixels[transformed..transformed + 4], &[10, 20, 30, 255]);
}

fn state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    let mut vertex = shader_create(11, 0, MATRIX_VERT); vertex[4] = 24;
    state.extend(vertex); state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend([vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0], vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688], vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17]].concat());
    state
}

fn matrix(values: [f32; 16]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 18), 0, 0]; words.extend(values.map(f32::to_bits)); words
}
