use super::super::super::completion::{PendingCompletion, WritableRegion};
use super::super::super::protocol::*;
use super::super::super::three_d::ResidentResource;
use super::super::super::VirtioGpu;
use super::super::virgl_draw_fixture::create;
use super::super::virgl_readback::{assert_response, read, resource_with_backing, transfer, virgl_context, SECOND_BACKING};
use super::super::header;
use super::resident_transactions::{drain, resource};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

#[test]
fn accepted_resident_readback_survives_unref_and_preserves_reused_id_owner() {
    for partial in [false, true] {
        let (mut gpu, mut mem, sequence) = pending(partial);
        let total = gpu.allocated_resource_bytes;
        assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, 1), RESP_OK_NODATA);
        assert_eq!(gpu.allocated_resource_bytes, total);
        let premature_release = gpu.take_3d_update();
        assert_response(&mut gpu, &mut mem, &create(1, 2, 1, 2, 4, 3), RESP_OK_NODATA);
        gpu.resources.get_mut(&1).unwrap().pixels.fill(37);
        let generation = gpu.virgl_contexts[&7].generation;
        gpu.resident_resources.insert(1, ResidentResource { context_id: 7, generation, producer_sequence: 72 });
        let pixels = vec![9; if partial { 4 } else { 48 }];
        assert!(gpu.complete_3d_readback(&mut mem, sequence, 1, &pixels));
        assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_OK_NODATA as u64));
        assert!(premature_release.is_empty(), "readback still owns its producer");
        assert_eq!(read(&mem, RAM_BASE, pixels.len().min(24)), pixels[..pixels.len().min(24)]);
        if !partial { assert_eq!(read(&mem, SECOND_BACKING, 24), pixels[24..]); }
        assert!(gpu.resources[&1].pixels.iter().all(|byte| *byte == 37));
        assert_eq!(gpu.resident_resources[&1].producer_sequence, 72);
        assert_eq!(gpu.allocated_resource_bytes, total);
        assert!(gpu.resource_lifetimes.retired.is_empty());
        assert_eq!(drain(&mut gpu), if partial { vec![71] } else { vec![] });
        assert!(!gpu.complete_3d_readback(&mut mem, sequence, 1, &pixels));
    }
}

#[test]
fn retired_resident_readback_failure_and_cancel_release_owner_losslessly() {
    for cancel in [false, true] {
        let (mut gpu, mut mem, sequence) = pending(true);
        assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, 1), RESP_OK_NODATA);
        gpu.resident_releases.extend(100..116);
        if cancel { gpu.cancel_3d(sequence); } else {
            assert!(gpu.complete_3d_readback(&mut mem, sequence, 99, &[9; 4]));
            assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_ERR_UNSPEC as u64));
        }
        assert_eq!(read(&mem, RAM_BASE, 4), [0; 4]);
        assert_eq!(gpu.allocated_resource_bytes, 0);
        assert!(gpu.resource_lifetimes.retired.is_empty());
        assert_eq!(gpu.pending_3d.len(), 1);
        assert_eq!(gpu.pending_3d_bytes, 12);
        let mut released = drain(&mut gpu); released.sort_unstable();
        assert_eq!(released, [71].into_iter().chain(100..116).collect::<Vec<_>>());
        assert_eq!(gpu.pending_3d_bytes, 0);
    }
}

#[test]
fn destroyed_and_recreated_context_cannot_complete_retained_resident_readback() {
    let (mut gpu, mut mem, sequence) = pending(true);
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
    assert!(gpu.take_3d_update().is_empty());
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert!(gpu.complete_3d_readback(&mut mem, sequence, 1, &[9; 4]));
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_ERR_UNSPEC as u64));
    assert_eq!(read(&mem, RAM_BASE, 4), [0; 4]);
    assert_eq!(drain(&mut gpu), [71]);
    assert!(gpu.resources.contains_key(&1));
    assert_eq!(gpu.allocated_resource_bytes, 48);
}

#[test]
fn retained_readback_owners_remain_charged_to_resident_count_and_bytes() {
    for (count, dimension) in [(16, 128), (4, 1024)] {
        let mut gpu = VirtioGpu::new();
        let mut mem = PhysicalMemory::new();
        assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
        let generation = gpu.virgl_contexts[&7].generation;
        let mut pending = Vec::new();
        for id in 100..100 + count {
            assert_response(&mut gpu, &mut mem, &create(id, 2, 1, 2, dimension, dimension), RESP_OK_NODATA);
            gpu.resident_resources.insert(id, ResidentResource { context_id: 7, generation, producer_sequence: id + 1000 });
            let mut command = transfer(0, 0, 1, 1, 0);
            command[56..60].copy_from_slice(&id.to_le_bytes());
            let job = gpu.execute_queued_command(&mut mem, &command).deferred.unwrap();
            pending.push(job.sequence);
            assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, id), RESP_OK_NODATA);
        }
        assert_eq!(gpu.resource_lifetimes.retained_residents.len(), count as usize);
        assert_eq!(gpu.allocated_resource_bytes, count as usize * dimension as usize * dimension as usize * 4);
        assert_response(&mut gpu, &mut mem, &create(1, 2, 1, 2, 128, 128), RESP_OK_NODATA);
        let rect = Rect { x: 0, y: 0, width: 128, height: 128 };
        assert!(!gpu.resident_target_eligible(1, rect));
        for sequence in pending { gpu.cancel_3d(sequence); }
        assert!(gpu.resident_target_eligible(1, rect));
        assert_eq!(gpu.allocated_resource_bytes, 128 * 128 * 4);
        assert!(gpu.resource_lifetimes.retained_residents.is_empty());
        assert!(gpu.resource_lifetimes.retired.is_empty());
    }
}

#[test]
fn accepted_readback_blocks_overwrite_until_completion_but_not_id_reuse() {
    let (mut gpu, mut mem, sequence) = pending(true);
    let mut upload = transfer(0, 0, 4, 3, 0);
    upload[..4].copy_from_slice(&CMD_TRANSFER_TO_HOST_3D.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &upload, RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.resources[&1].pixels.iter().all(|byte| *byte == 0));
    assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, 1), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &create(1, 2, 1, 2, 4, 3), RESP_OK_NODATA);
    let rect = Rect { x: 0, y: 0, width: 4, height: 3 };
    assert!(gpu.resident_overwrite_allowed(1, rect), "old generation does not block the new allocation");
    gpu.cancel_3d(sequence);
    assert_eq!(drain(&mut gpu), [71]);
}

pub(super) fn pending(partial: bool) -> (VirtioGpu, PhysicalMemory, u32) {
    let (mut gpu, mut mem) = resource_with_backing();
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    gpu.resident_resources.insert(1, ResidentResource {
        context_id: 7, generation: gpu.virgl_contexts[&7].generation, producer_sequence: 71,
    });
    let command = if partial { transfer(0, 0, 1, 1, 0) } else { transfer(0, 0, 4, 3, 0) };
    let deferred = gpu.execute_queued_command(&mut mem, &command).deferred.unwrap();
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion {
        header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }],
        used: RAM_BASE + 0x7100, queue_size: 8, head: 1,
    }));
    assert_eq!(gpu.take_3d_update().get(..4), Some(&b"VGR1"[..]));
    (gpu, mem, deferred.sequence)
}
