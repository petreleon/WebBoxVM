use super::super::super::protocol::*;
use super::super::super::completion::{PendingCompletion, WritableRegion};
use super::super::virgl_resident_sample::{draw_command, prepared_sample};
use super::super::virgl_draw_fixture::{assert_response, create, draw, submit, word, TARGET};
use super::super::virgl_readback::transfer;
use super::resident_transactions::{resource, drain};
use crate::constants::RAM_BASE;

#[test]
fn resident_readback_excludes_sampling_and_copy_in_both_admission_orders() {
    for readback_first in [true, false] {
        let (mut gpu, mut mem) = prepared_sample();
        let mut readback = transfer(0, 0, 1, 1, 0);
        readback[56..60].copy_from_slice(&8u32.to_le_bytes());
        let sample = submit(&draw_command());
        let (first, second) = if readback_first { (&readback, &sample) } else { (&sample, &readback) };
        let pending = gpu.execute_queued_command(&mut mem, first).deferred.unwrap();
        let bytes = gpu.pending_3d_bytes;
        assert_response(&mut gpu, &mut mem, second, RESP_ERR_INVALID_PARAMETER);
        assert_eq!(gpu.pending_3d.len(), 1);
        assert_eq!(gpu.pending_3d_bytes, bytes);
        if readback_first {
            assert_response(&mut gpu, &mut mem, &create(9, 2, 1, 2, 65, 65), RESP_OK_NODATA);
            assert_response(&mut gpu, &mut mem, &resource(CMD_CTX_ATTACH_RESOURCE, 9), RESP_OK_NODATA);
            let copy = submit(&[word(17, 0, 13), 9, 0, 0, 0, 0, 8, 0, 0, 0, 0, 65, 65, 1]);
            assert_response(&mut gpu, &mut mem, &copy, RESP_ERR_INVALID_PARAMETER);
        }
        gpu.cancel_3d(pending.sequence);
    }
}

#[test]
fn resident_samples_reject_multiple_works_before_creating_lifetime_pins() {
    let (mut gpu, mut mem) = prepared_sample();
    let mut words = draw_command(); words.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&words), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.pending_3d.is_empty());
    assert_eq!(gpu.pending_3d_bytes, 0);
    assert_eq!(gpu.resident_resources[&8].producer_sequence, 71);
    assert!(!gpu.resident_resources.contains_key(&TARGET));
}

#[test]
fn final_retained_owner_release_reuses_only_one_slot_when_other_jobs_fill_queue() {
    let (mut gpu, mut mem, sequence) = super::resident_readback_lifetime::pending(true);
    assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, 1), RESP_OK_NODATA);
    for id in 100..115 {
        assert_response(&mut gpu, &mut mem, &create(id, 2, 1, 2, 65, 65), RESP_OK_NODATA);
        gpu.resident_resources.insert(id, super::super::super::three_d::ResidentResource {
            context_id: 7, generation: gpu.virgl_contexts[&7].generation, producer_sequence: id + 1000,
        });
        let mut command = transfer(0, 0, 1, 1, 0);
        command[56..60].copy_from_slice(&id.to_le_bytes());
        let pending = gpu.execute_queued_command(&mut mem, &command).deferred.unwrap();
        assert!(gpu.attach_3d_completion(pending.sequence, PendingCompletion {
            header: pending.header, output: vec![WritableRegion { addr: RAM_BASE + 0x8000, len: 24 }],
            used: RAM_BASE + 0x9000, queue_size: 32, head: 0,
        }));
    }
    gpu.resident_releases.extend(200..216);
    assert_eq!(gpu.pending_3d.len(), 16);
    assert!(gpu.complete_3d_readback(&mut mem, sequence, 99, &[9; 4]));
    assert_eq!(gpu.pending_3d.len(), 16);
    assert_eq!(gpu.pending_3d_bytes, 15 * 40 + 12);
    for _ in 0..15 { assert_eq!(gpu.take_3d_update().get(..4), Some(&b"VGR1"[..])); }
    assert_eq!(drain(&mut gpu), [71].into_iter().chain(200..216).collect::<Vec<_>>());
    assert_eq!(gpu.pending_3d.len(), 15);
    assert_eq!(gpu.pending_3d_bytes, 15 * 40);
}
