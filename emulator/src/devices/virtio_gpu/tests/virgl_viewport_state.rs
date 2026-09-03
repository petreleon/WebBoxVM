use super::super::protocol::{RESP_ERR_INVALID_PARAMETER, RESP_OK_NODATA};
use super::virgl_draw::{
    FRAG, TARGET, VERT, assert_response, clear, draw, framebuffer, prepared, shader_bind,
    shader_create, submit, surface_create, upload_vertices, vertex_state, word,
};
use super::{virgl_source_over_state, virgl_viewport_scissor_state};

#[test]
fn draw_requires_the_standard_viewport_rasterizer_and_scissor_chain() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_vertices(&mut gpu);
    let mut draw_without_state = clear([0.1, 0.2, 0.3, 1.0]);
    draw_without_state.extend(draw());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&draw_without_state),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&virgl_viewport_scissor_state(14)),
        RESP_OK_NODATA,
    );
    let mut enabled = clear([0.1, 0.2, 0.3, 1.0]);
    enabled.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&enabled), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&[word(2, 2, 1), 0]),
        RESP_OK_NODATA,
    );
    let mut draw_without_rasterizer = clear([0.1, 0.2, 0.3, 1.0]);
    draw_without_rasterizer.extend(draw());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&draw_without_rasterizer),
        RESP_ERR_INVALID_PARAMETER,
    );
}
