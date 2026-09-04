use super::super::protocol::*;
use super::{
    header, response_type, virgl_draw_fixture::*, virgl_source_over_state,
    virgl_viewport_scissor_state,
};

const UNIFORM: u32 = 7;
const MATRIX_VERT: &str = "VERT\nDCL IN[0]\nDCL CONST[0..3]\nDCL OUT[0], POSITION\nDP4 OUT[0].x, IN[0], CONST[0]\nDP4 OUT[0].y, IN[0], CONST[1]\nDP4 OUT[0].z, IN[0], CONST[2]\nDP4 OUT[0].w, IN[0], CONST[3]\nEND\n";
const MATRIX: [f32; 16] = [
    0.5, 0.0, 0.0, 0.25,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[test]
fn vertex_matrix_uniform_buffer_snapshots_the_standard_64_byte_range() {
    let (mut gpu, mut mem) = prepared_nonresident();
    attach(&mut gpu, &mut mem); configure(&mut gpu, &mut mem); upload_vertices(&mut gpu); store(&mut gpu, MATRIX);
    assert_response(&mut gpu, &mut mem, &submit(&uniform(0, 64)), RESP_OK_NODATA);
    let packet = render(&mut gpu, &mut mem).expect("matrix uniform draw");
    assert_eq!(read_u32(&packet, 4), Some(15));
    assert_eq!([56, 60, 64, 68].map(|at| read_u32(&packet, at)), [MATRIX[0], MATRIX[1], MATRIX[2], MATRIX[3]].map(f32::to_bits).map(Some));
    assert_eq!([120, 124, 128, 132].map(|at| read_u32(&packet, at)), [0.0, 0.75, 0.0, 1.0].map(f32::to_bits).map(Some));
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("matrix uniform effect"); assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize; let transformed = ((400 * 1024 + 540) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[77, 51, 26, 255]);
    assert_eq!(&gpu.resources[&TARGET].pixels[transformed..transformed + 4], &[58, 102, 20, 255]);
}

#[test]
fn malformed_matrix_uniform_binding_preserves_the_prior_matrix() {
    let (mut gpu, mut mem) = prepared_nonresident();
    attach(&mut gpu, &mut mem); configure(&mut gpu, &mut mem); upload_vertices(&mut gpu); store(&mut gpu, MATRIX);
    assert_response(&mut gpu, &mut mem, &submit(&uniform(0, 64)), RESP_OK_NODATA);
    for words in [uniform(2, 64), uniform(0, 32), uniform(4, 64)] {
        assert_response(&mut gpu, &mut mem, &submit(&words), RESP_ERR_INVALID_PARAMETER);
    }
    let packet = render(&mut gpu, &mut mem).expect("preserved matrix uniform");
    assert_eq!([4, 56].map(|at| read_u32(&packet, at)), [Some(15), Some(MATRIX[0].to_bits())]);
}

fn attach(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [UNIFORM, 0, 64, 1 << 6, 64, 1, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut command = header(CMD_CTX_ATTACH_RESOURCE); for value in [UNIFORM, 0] { push_u32(&mut command, value); }
    assert_response(gpu, mem, &command, RESP_OK_NODATA);
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9)); state.extend(shader_create(11, 0, MATRIX_VERT));
    state.extend(shader_create(12, 1, FRAG)); state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn store(gpu: &mut super::super::VirtioGpu, matrix: [f32; 16]) {
    let bytes: Vec<u8> = matrix.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

fn uniform(offset: u32, bytes: u32) -> Vec<u32> {
    vec![word(27, 0, 5), 0, 0, offset, bytes, UNIFORM]
}

fn render(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) -> Option<Vec<u8>> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    (response_type(&gpu.execute_command(mem, &submit(&command))) == RESP_OK_NODATA).then(|| gpu.take_3d_update())
}
