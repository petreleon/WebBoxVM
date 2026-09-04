use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::three_d::{BrowserCompletion, ResidentResource};
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::constants::RAM_BASE;

#[test]
fn singleton_solid_redraw_rekeys_a_resident_batch_target() {
    let (mut gpu, mut mem) = prepared();
    configure_solid(&mut gpu, &mut mem); upload_vertices(&mut gpu);
    let command = draw_command();
    let first = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("solid singleton");
    attach(&mut gpu, first.sequence, first.header, 1);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGB1");
    assert_eq!([4, 20, 24].map(|at| read_u32(&packet, at)), [Some(6), Some(1), Some(1)]);
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    let second = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("solid replacement");
    attach(&mut gpu, second.sequence, second.header, 2);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 48].map(|at| read_u32(&packet, at)), [Some(7), Some(first.sequence)]);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, second.sequence);
}

#[test]
fn singleton_textured_redraw_rekeys_a_resident_material_target() {
    let (mut gpu, mut mem) = prepared();
    configure_texture(&mut gpu, &mut mem); upload_textured_vertices(&mut gpu);
    let command = draw_command();
    let first = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("textured singleton");
    attach(&mut gpu, first.sequence, first.header, 1);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGM1");
    assert_eq!([4, 20, 24].map(|at| read_u32(&packet, at)), [Some(2), Some(1), Some(2)]);
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    let second = gpu.execute_queued_command(&mut mem, &submit(&command)).deferred.expect("textured replacement");
    attach(&mut gpu, second.sequence, second.header, 2);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 48].map(|at| read_u32(&packet, at)), [Some(3), Some(first.sequence)]);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, second.sequence);
}

#[test]
fn resident_target_budget_rejects_a_fifth_four_mebibyte_target() {
    let (mut gpu, mut mem) = prepared();
    let generation = gpu.virgl_contexts[&7].generation;
    for id in 100..104 {
        assert_response(&mut gpu, &mut mem, &create(id, 2, 1, 2, 1024, 1024), RESP_OK_NODATA);
        gpu.resident_resources.insert(id, ResidentResource { context_id: 7, generation, producer_sequence: id });
    }
    assert_response(&mut gpu, &mut mem, &create(104, 2, 1, 2, 1024, 1024), RESP_OK_NODATA);
    assert!(!gpu.resident_target_eligible(104, Rect { x: 0, y: 0, width: 1024, height: 1024 }));
}

fn configure_solid(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn configure_texture(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT)); state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(textured_vertex_state());
    state.extend([vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0], vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688], vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17]].concat());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn draw_command() -> Vec<u32> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw()); command
}

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, head: u16) {
    assert!(gpu.attach_3d_completion(sequence, PendingCompletion {
        header, output: vec![WritableRegion { addr: RAM_BASE + u64::from(head) * 0x100, len: 24 }],
        used: RAM_BASE + 0x7000, queue_size: 8, head,
    }));
}
