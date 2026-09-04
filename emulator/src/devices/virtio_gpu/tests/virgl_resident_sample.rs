use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::three_d::{BrowserCompletion, ResidentResource};
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::constants::RAM_BASE;

const SOURCE: u32 = 8;
const WIDTH: u32 = 65;
const HEIGHT: u32 = 65;

#[test]
fn resident_texture_sampling_keeps_both_cpu_shadows_unread() {
    let (mut gpu, mut mem) = prepared_sample();
    let source_shadow = gpu.resources[&SOURCE].pixels.clone();
    let target_shadow = gpu.resources[&TARGET].pixels.clone();
    let deferred = gpu
        .execute_queued_command(&mut mem, &submit(&draw_command()))
        .deferred
        .expect("resident sample defers");
    attach(&mut gpu, deferred.sequence, deferred.header, 1);
    let mut detach = header(CMD_CTX_DETACH_RESOURCE);
    for value in [SOURCE, 0] {
        push_u32(&mut detach, value);
    }
    assert_response(&mut gpu, &mut mem, &detach, RESP_ERR_INVALID_PARAMETER);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&draw_command()),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.resident_sample_in_flight(SOURCE));
    assert_eq!(
        gpu.pending_3d[0].browser_completion,
        BrowserCompletion::Resident
    );
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGM1");
    assert_eq!(packet.len(), 188);
    assert_eq!(
        [4, 8, 20, 24, 48, 52, 56, 100, 104, 108, 112].map(|at| read_u32(&packet, at)),
        [
            Some(12),
            Some(deferred.sequence),
            Some(1),
            Some(2),
            Some(3),
            Some(0),
            Some(3),
            Some(0x1092),
            Some(WIDTH),
            Some(HEIGHT),
            Some(71),
        ]
    );
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert_eq!(gpu.resident_resources[&SOURCE].producer_sequence, 71);
    assert_eq!(
        gpu.resident_resources[&TARGET].producer_sequence,
        deferred.sequence
    );
    assert_eq!(gpu.resources[&SOURCE].pixels, source_shadow);
    assert_eq!(gpu.resources[&TARGET].pixels, target_shadow);
    assert!(!gpu.resident_sample_in_flight(SOURCE));
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
}

#[test]
fn stale_resident_texture_sample_releases_only_its_new_target() {
    let (mut gpu, mut mem) = prepared_sample();
    let deferred = gpu
        .execute_queued_command(&mut mem, &submit(&draw_command()))
        .deferred
        .expect("resident sample defers");
    attach(&mut gpu, deferred.sequence, deferred.header, 2);
    let _ = gpu.take_3d_update();
    gpu.forget_resident(SOURCE);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(71));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(deferred.sequence));
    assert_eq!(mem.read(RAM_BASE + 0x7100, 4), Some(RESP_ERR_UNSPEC as u64));
}

fn prepared_sample() -> (super::super::VirtioGpu, crate::memory::PhysicalMemory) {
    let (mut gpu, mut mem) = prepared();
    assert_response(
        &mut gpu,
        &mut mem,
        &create(SOURCE, 2, 1, 10, WIDTH, HEIGHT),
        RESP_OK_NODATA,
    );
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [SOURCE, 0] {
        push_u32(&mut attach, value);
    }
    assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    let generation = gpu.virgl_contexts[&7].generation;
    gpu.resident_resources.insert(
        SOURCE,
        ResidentResource {
            context_id: 7,
            generation,
            producer_sequence: 71,
        },
    );
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend(
        [
            vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0],
            vec![word(1, 6, 6), 18, SOURCE, 1, 0, 0, 0x688],
            vec![word(10, 0, 3), 1, 0, 18],
            vec![word(18, 0, 3), 1, 0, 17],
        ]
        .concat(),
    );
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    (gpu, mem)
}

fn draw_command() -> Vec<u32> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    command
}

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, head: u16) {
    assert!(gpu.attach_3d_completion(
        sequence,
        PendingCompletion {
            header,
            output: vec![WritableRegion {
                addr: RAM_BASE + 0x6f00 + u64::from(head) * 0x100,
                len: 24
            }],
            used: RAM_BASE + 0x7200,
            queue_size: 8,
            head,
        }
    ));
}
