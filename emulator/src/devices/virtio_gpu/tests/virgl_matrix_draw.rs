use super::super::protocol::*;
use super::{
    response_type, virgl_draw_fixture::*, virgl_source_over_state,
    virgl_viewport_scissor_state,
};

const MATRIX_VERT: &str = "VERT\nDCL IN[0]\nDCL CONST[0..3]\nDCL OUT[0], POSITION\nDP4 OUT[0].x, IN[0], CONST[0]\nDP4 OUT[0].y, IN[0], CONST[1]\nDP4 OUT[0].z, IN[0], CONST[2]\nDP4 OUT[0].w, IN[0], CONST[3]\nEND\n";
const MATRIX: [f32; 16] = [
    0.5, 0.0, 0.0, 0.25,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[test]
fn vertex_dp4_matrix_transforms_standard_virgl_vertices() {
    let (mut gpu, mut mem) = prepared_nonresident();
    configure(&mut gpu, &mut mem);
    upload_vertices(&mut gpu);
    assert_response(&mut gpu, &mut mem, &submit(&matrix(MATRIX)), RESP_OK_NODATA);
    let packet = render(&mut gpu, &mut mem).expect("matrix draw");
    assert_eq!(read_u32(&packet, 4), Some(15));
    assert_eq!(
        [56, 60, 64, 68].map(|offset| read_u32(&packet, offset)),
        [MATRIX[0], MATRIX[1], MATRIX[2], MATRIX[3]].map(f32::to_bits).map(Some),
    );
    assert_eq!([120, 124, 128, 132].map(|offset| read_u32(&packet, offset)), [0.0, 0.75, 0.0, 1.0].map(f32::to_bits).map(Some));
    let effect = gpu.pending_3d[0].effect.clone().expect("matrix effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    let transformed = ((400 * 1024 + 540) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[77, 51, 26, 255]);
    assert_eq!(&gpu.resources[&TARGET].pixels[transformed..transformed + 4], &[58, 102, 20, 255]);
}

#[test]
fn matrix_draw_fails_closed_for_nonprojectable_vertices() {
    let (mut gpu, mut mem) = prepared_nonresident();
    configure(&mut gpu, &mut mem);
    upload_vertices(&mut gpu);
    let mut invalid = MATRIX; invalid[15] = 0.0;
    assert_response(&mut gpu, &mut mem, &submit(&matrix(invalid)), RESP_OK_NODATA);
    assert!(render(&mut gpu, &mut mem).is_none());
    assert!(gpu.pending_3d.is_empty());
}

#[test]
fn matrix_constants_require_one_finite_vertex_stage_matrix() {
    let (mut gpu, mut mem) = prepared_nonresident();
    for words in [
        vec![word(12, 0, 3), 0, 0, 0],
        matrix(MATRIX.map(|value| value).map(|value| if value == 1.0 { f32::NAN } else { value })),
    ] {
        assert_response(&mut gpu, &mut mem, &submit(&words), RESP_ERR_INVALID_PARAMETER);
    }
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, MATRIX_VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn matrix(values: [f32; 16]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 18), 0, 0]; words.extend(values.map(f32::to_bits)); words
}

fn render(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) -> Option<Vec<u8>> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    (response_type(&gpu.execute_command(mem, &submit(&command))) == RESP_OK_NODATA).then(|| gpu.take_3d_update())
}
