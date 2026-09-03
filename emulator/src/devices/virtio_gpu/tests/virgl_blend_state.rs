use super::super::protocol::{RESP_ERR_INVALID_PARAMETER, RESP_OK_NODATA};
use super::virgl_draw::{
    FRAG, TARGET, VERT, assert_response, clear, draw, framebuffer, prepared, shader_bind,
    shader_create, submit, surface_create, upload_vertices, vertex_state, word,
};
use super::virgl_source_over_state;

#[test]
fn draw_rejects_a_source_over_state_after_standard_unbind() {
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
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&[word(2, 1, 1), 0]),
        RESP_OK_NODATA,
    );
    upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&command),
        RESP_ERR_INVALID_PARAMETER,
    );
}
