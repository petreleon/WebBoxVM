use super::super::protocol::*;
use super::{
    response_type, virgl_draw_fixture::*, virgl_source_over_state,
    virgl_viewport_scissor_state,
};

const VERT: &str = "VERT\nDCL IN[0], POSITION\nDCL IN[1], GENERIC[0]\nDCL CONST[0..3]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[1], IN[1]\nDP4 OUT[0].w, IN[0], CONST[3]\nDP4 OUT[0].x, IN[0], CONST[0]\nDP4 OUT[0].z, IN[0], CONST[2]\nDP4 OUT[0].y, IN[0], CONST[1]\nEND\n";
const FRAGMENT: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL OUT[0], COLOR[0]\nMOV OUT[0], IN[0]\nEND\n";
const MATRIX: [f32; 16] = [
    0.5, 0.0, 0.0, 0.25,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[test]
fn matrix_generic_vertex_colors_keep_raw_attributes_for_webgpu() {
    let (mut gpu, mut mem) = prepared_nonresident();
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA); upload(&mut gpu);
    assert_response(&mut gpu, &mut mem, &submit(&matrix(MATRIX)), RESP_OK_NODATA);
    let packet = render(&mut gpu, &mut mem).expect("matrix vertex-color draw");
    assert_eq!([4, 56, 120, 136].map(|at| read_u32(&packet, at)), [Some(16), Some(MATRIX[0].to_bits()), Some(0), Some(1.0f32.to_bits())]);
    let effect = gpu.pending_3d[0].effect.clone().expect("matrix vertex-color effect"); assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize; let transformed = ((400 * 1024 + 540) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[77, 51, 26, 255]);
    assert_ne!(&gpu.resources[&TARGET].pixels[transformed..transformed + 4], &[77, 51, 26, 255]);
}

fn state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    let mut vertex = shader_create(11, 0, VERT); vertex[4] = 24; state.extend(vertex);
    let mut fragment = shader_create(12, 1, FRAGMENT); fragment[4] = 11; state.extend(fragment);
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_color_state()); state
}

fn vertex_color_state() -> Vec<u32> {
    [vec![word(1, 5, 9), 10, 0, 0, 0, 31, 16, 0, 0, 31], vec![word(2, 5, 1), 10], vec![word(6, 0, 3), 32, 0, BUFFER]].concat()
}

fn upload(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
        -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

fn matrix(values: [f32; 16]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 18), 0, 0]; words.extend(values.map(f32::to_bits)); words
}

fn render(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) -> Option<Vec<u8>> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    (response_type(&gpu.execute_command(mem, &submit(&command))) == RESP_OK_NODATA).then(|| gpu.take_3d_update())
}
