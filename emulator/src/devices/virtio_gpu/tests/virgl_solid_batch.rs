use super::super::protocol::*;
use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::three_d::BrowserCompletion;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
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
    assert_eq!([4, 12, 16, 20, 24].map(|offset| read_u32(&packet, offset)), [Some(6), Some(1024), Some(768), Some(2), Some(1)]);
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
fn solid_batch_accepts_a_bounded_gpu_resident_completion() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5])); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("solid batch defers");
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion { header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }], used: RAM_BASE + 0x7100, queue_size: 8, head: 1 }));
    assert_eq!(&gpu.take_3d_update()[..4], b"VGB1");
    gpu.pending_damage = Some(Rect { x: 0, y: 0, width: 1024, height: 768 });
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    let resident = gpu.resident_resources.get(&TARGET).expect("resident target");
    assert_eq!(resident.producer_sequence, deferred.sequence);
    assert!(gpu.pending_damage.is_none());
    let mut flush = header(CMD_RESOURCE_FLUSH);
    for value in [0, 0, 1024, 768, TARGET] { push_u32(&mut flush, value); }
    push_u32(&mut flush, 0);
    assert_response(&mut gpu, &mut mem, &flush, RESP_OK_NODATA);
    assert!(gpu.take_scanout_update().is_empty());
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
}

#[test]
fn resident_solid_batch_reuses_its_browser_target_on_the_next_full_redraw() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5])); command.extend(draw());
    let first = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("first batch");
    attach(&mut gpu, first.sequence, first.header, RAM_BASE + 0x7000, 1);
    assert_eq!(read_u32(&gpu.take_3d_update(), 4), Some(6));
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    let second = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("replacement batch");
    attach(&mut gpu, second.sequence, second.header, RAM_BASE + 0x7100, 2);
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 48].map(|offset| read_u32(&packet, offset)), [Some(7), Some(first.sequence)]);
    assert_eq!(packet.len(), 268);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, second.sequence);
    assert!(gpu.take_3d_update().is_empty());
}

#[test]
fn cpu_authority_epoch_rejects_an_unresolved_resident_batch() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5])); command.extend(draw());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("resident batch");
    attach(&mut gpu, deferred.sequence, deferred.header, RAM_BASE + 0x7000, 1);
    let _ = gpu.take_3d_update();
    gpu.forget_resident(TARGET);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(deferred.sequence));
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_ERR_UNSPEC as u64));
}

#[test]
fn stale_resident_redraw_releases_its_new_browser_target_fail_closed() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut command = clear([0.0, 0.0, 0.0, 1.0]);
    command.extend(constants([1.0, 0.0, 0.0, 0.5])); command.extend(draw());
    command.extend(constants([0.0, 1.0, 0.0, 0.5])); command.extend(draw());
    let first = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("first batch");
    attach(&mut gpu, first.sequence, first.header, RAM_BASE + 0x7000, 1);
    let _ = gpu.take_3d_update();
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    let second = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("replacement batch");
    attach(&mut gpu, second.sequence, second.header, RAM_BASE + 0x7100, 2);
    let _ = gpu.take_3d_update();
    gpu.forget_resident(TARGET);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(first.sequence));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(second.sequence));
    assert_eq!(mem.read(RAM_BASE + 0x7100, 4), Some(RESP_ERR_UNSPEC as u64));
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

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, address: u64, head: u16) {
    assert!(gpu.attach_3d_completion(sequence, PendingCompletion {
        header, output: vec![WritableRegion { addr: address, len: 24 }], used: RAM_BASE + 0x7200,
        queue_size: 8, head,
    }));
}

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut command = vec![word(12, 0, 6), 1, 0];
    command.extend(color.map(f32::to_bits));
    command
}
