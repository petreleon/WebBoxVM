use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::{RESP_ERR_INVALID_PARAMETER, RESP_OK_NODATA};
use super::super::three_d::BrowserCompletion;
use super::virgl_draw::{
    FRAG, TARGET, VERT, assert_response, clear, draw, framebuffer, prepared, shader_bind,
    shader_create, submit, surface_create, upload_vertices, vertex_state, word,
};
use super::{virgl_source_over_state, virgl_viewport_scissor_state};
use crate::constants::RAM_BASE;

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
    state.extend(virgl_viewport_scissor_state(14));
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

#[test]
fn opaque_blend_uses_a_readback_replace_batch() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(opaque_state(13, 15));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("opaque batch");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Readback);
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion { header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head: 1 }));
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1"); assert_eq!([4, 20, 24].map(|offset| super::super::protocol::read_u32(&packet, offset)), [Some(8), Some(1), Some(0)]);
    assert!(gpu.complete_3d_readback(&mut mem, deferred.sequence, 2, &[1, 2, 3, 255].repeat(1024 * 768)));
    assert_eq!(&gpu.resources[&TARGET].pixels[..4], &[3, 2, 1, 255]);
}

#[test]
fn rgb_opaque_blend_uses_a_readback_masked_batch() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(opaque_state(13, 7));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("RGB opaque batch");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Readback);
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion { header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head: 1 }));
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1"); assert_eq!([4, 20, 24].map(|offset| super::super::protocol::read_u32(&packet, offset)), [Some(10), Some(1), Some(0)]);
    assert!(gpu.complete_3d_readback(&mut mem, deferred.sequence, 2, &[1, 2, 3, 255].repeat(1024 * 768)));
}

#[test]
fn blend_mixed_batch_fails_without_committing_its_binding() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(opaque_state(14, 15)); state.extend(virgl_viewport_scissor_state(15)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut mixed = clear([0.1, 0.2, 0.3, 1.0]); mixed.extend(draw()); mixed.extend([word(2, 1, 1), 13]); mixed.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&mixed), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.pending_3d.is_empty()); assert!(gpu.take_3d_update().is_empty());
    let mut opaque = clear([0.1, 0.2, 0.3, 1.0]); opaque.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&opaque)).deferred.expect("opaque binding survives");
    assert_eq!(super::super::protocol::read_u32(&gpu.take_3d_update(), 4), Some(8));
    assert_eq!(gpu.pending_3d[0].sequence, deferred.sequence);
}

fn opaque_state(handle: u32, mask: u32) -> Vec<u32> {
    let mut words = vec![word(1, 1, 11), handle, 0, 0, mask << 27];
    words.extend([0; 7]); words.extend([word(2, 1, 1), handle]); words
}
