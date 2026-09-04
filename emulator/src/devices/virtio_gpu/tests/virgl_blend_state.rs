use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER, RESP_OK_NODATA, read_u32};
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
fn opaque_blend_retains_and_rekeys_an_eligible_full_target() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(opaque_state(13, 15));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    let first = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("opaque batch");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    attach(&mut gpu, first.sequence, first.header, RAM_BASE + 0x7000, 1);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1"); assert_eq!([4, 20, 24].map(|at| read_u32(&packet, at)), [Some(14), Some(1), Some(15)]);
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, first.sequence);
    let second = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("opaque rekey");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    attach(&mut gpu, second.sequence, second.header, RAM_BASE + 0x7200, 2);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 8, 24, 48].map(|at| read_u32(&packet, at)), [Some(15), Some(second.sequence), Some(15), Some(first.sequence)]);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, second.sequence);
}

#[test]
fn rgb_opaque_blend_retains_its_exact_write_mask() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(opaque_state(13, 7));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("RGB opaque batch");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    attach(&mut gpu, deferred.sequence, deferred.header, RAM_BASE + 0x7000, 1);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1"); assert_eq!([4, 20, 24].map(|at| read_u32(&packet, at)), [Some(14), Some(1), Some(7)]);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, deferred.sequence);
}

#[test]
fn partial_opaque_blend_retains_its_standard_channel_mask() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9)); state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(opaque_state(13, 9));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("masked opaque batch");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    attach(&mut gpu, deferred.sequence, deferred.header, RAM_BASE + 0x7000, 1);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1"); assert_eq!([4, 20, 24].map(|at| read_u32(&packet, at)), [Some(14), Some(1), Some(9)]);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, deferred.sequence);
}

#[test]
fn zero_color_mask_fails_stream_validation() {
    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &submit(&opaque_state(13, 0)), RESP_ERR_INVALID_PARAMETER);
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
    assert_eq!(read_u32(&gpu.take_3d_update(), 4), Some(14));
    assert_eq!(gpu.pending_3d[0].sequence, deferred.sequence);
}

fn opaque_state(handle: u32, mask: u32) -> Vec<u32> {
    let mut words = vec![word(1, 1, 11), handle, 0, 0, mask << 27];
    words.extend([0; 7]); words.extend([word(2, 1, 1), handle]); words
}

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, address: u64, head: u16) {
    assert!(gpu.attach_3d_completion(sequence, PendingCompletion {
        header, output: vec![WritableRegion { addr: address, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head,
    }));
}
