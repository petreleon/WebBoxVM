use super::super::protocol::*;
use super::super::completion::{PendingCompletion, WritableRegion};
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::constants::RAM_BASE;

const CONSTANT_FRAG: &str =
    "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn standard_solid_draws_batch_in_order_after_one_clear() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5]));
    command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5]));
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1");
    assert_eq!([4, 12, 16, 20, 24].map(|offset| read_u32(&packet, offset)), [Some(1), Some(1024), Some(768), Some(2), Some(0)]);
    assert_eq!(packet.len(), 264);
    assert_eq!([48, 156].map(|offset| read_u32(&packet, offset)), [Some(3), Some(3)]);
    let effect = gpu.pending_3d[0].effect.clone().expect("solid batch effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[0, 128, 64, 255]);
}

#[test]
fn solid_batch_accepts_gpu_color_readback_after_browser_delivery() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5])); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("solid batch defers");
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion { header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head: 1 }));
    assert_eq!(&gpu.take_3d_update()[..4], b"VGB1");
    let pixels = [1, 2, 3, 255].repeat(1024 * 768);
    assert!(gpu.complete_3d_readback(&mut mem, deferred.sequence, 2, &pixels));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[middle..middle + 4], &[3, 2, 1, 255]);
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
}

#[test]
fn solid_batch_caps_draw_count_without_committing_a_packet() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5]));
    for _ in 0..17 { command.extend(draw()); }
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.take_3d_update().is_empty());
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, CONSTANT_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend([word(1, 0, 5), 15, 7, 0, 0, 0, word(2, 0, 1), 15, word(2, 0, 1), 0]);
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
    upload_vertices(gpu);
}

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut command = vec![word(12, 0, 6), 1, 0];
    command.extend(color.map(f32::to_bits));
    command
}
