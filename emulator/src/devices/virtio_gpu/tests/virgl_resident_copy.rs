use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::three_d::{BrowserCompletion, ResidentResource};
use super::{header, virgl_draw_fixture::{assert_response, create, prepared, submit, word}};
use crate::constants::RAM_BASE;

const SOURCE: u32 = 8;
const TARGET: u32 = 9;
const WIDTH: u32 = 65;
const HEIGHT: u32 = 65;

#[test]
fn full_resident_copy_keeps_its_source_and_promotes_a_fresh_target() {
    let (mut gpu, mut mem) = resident_pair();
    let source_shadow = gpu.resources[&SOURCE].pixels.clone();
    let target_shadow = gpu.resources[&TARGET].pixels.clone();
    gpu.pending_damage = Some(Rect { x: 3, y: 2, width: 1, height: 1 });
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&copy(0, 0, 0, 0, WIDTH, HEIGHT)))
        .deferred.expect("resident copy defers");
    assert_response(&mut gpu, &mut mem, &submit(&copy(0, 0, 0, 0, WIDTH, HEIGHT)), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.resident_copy_in_flight(SOURCE)); assert!(gpu.resident_copy_in_flight(TARGET));
    attach(&mut gpu, deferred.sequence, deferred.header, 1);
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VRC1");
    assert_eq!(packet.len(), 24);
    assert_eq!([4, 8, 12, 16, 20].map(|at| read_u32(&packet, at)),
        [Some(1), Some(deferred.sequence), Some(71), Some(WIDTH), Some(HEIGHT)]);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert_eq!(gpu.resident_resources[&SOURCE].producer_sequence, 71);
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, deferred.sequence);
    assert_eq!(gpu.resources[&SOURCE].pixels, source_shadow);
    assert_eq!(gpu.resources[&TARGET].pixels, target_shadow);
    assert_eq!(gpu.pending_damage, Some(Rect { x: 3, y: 2, width: 1, height: 1 }));
    assert!(!gpu.resident_copy_in_flight(SOURCE)); assert!(!gpu.resident_copy_in_flight(TARGET));
    assert!(gpu.take_3d_update().is_empty());
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
}

#[test]
fn resident_copy_rejects_partial_regions_and_releases_a_stale_target() {
    let (mut gpu, mut mem) = resident_pair();
    assert_response(&mut gpu, &mut mem, &submit(&copy(0, 0, 0, 0, WIDTH - 1, HEIGHT)), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.pending_3d.is_empty());
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&copy(0, 0, 0, 0, WIDTH, HEIGHT)))
        .deferred.expect("resident copy defers");
    attach(&mut gpu, deferred.sequence, deferred.header, 2);
    assert_eq!(&gpu.take_3d_update()[..4], b"VRC1");
    gpu.forget_resident(SOURCE);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(71));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(deferred.sequence));
    assert_eq!(mem.read(RAM_BASE + 0x7100, 4), Some(RESP_ERR_UNSPEC as u64));
}

fn resident_pair() -> (super::super::VirtioGpu, crate::memory::PhysicalMemory) {
    let (mut gpu, mut mem) = prepared();
    for resource in [SOURCE, TARGET] {
        assert_response(&mut gpu, &mut mem, &create(resource, 2, 1, 2, WIDTH, HEIGHT), RESP_OK_NODATA);
        let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
        for value in [resource, 0] { push_u32(&mut attach, value); }
        assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    }
    let generation = gpu.virgl_contexts[&7].generation;
    gpu.resident_resources.insert(SOURCE, ResidentResource { context_id: 7, generation, producer_sequence: 71 });
    (gpu, mem)
}

fn copy(dst_x: u32, dst_y: u32, src_x: u32, src_y: u32, width: u32, height: u32) -> Vec<u32> {
    vec![word(17, 0, 13), TARGET, 0, dst_x, dst_y, 0, SOURCE, 0, src_x, src_y, 0, width, height, 1]
}

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, head: u16) {
    assert!(gpu.attach_3d_completion(sequence, PendingCompletion {
        header, output: vec![WritableRegion { addr: RAM_BASE + 0x6f00 + u64::from(head) * 0x100, len: 24 }],
        used: RAM_BASE + 0x7200, queue_size: 8, head,
    }));
}
