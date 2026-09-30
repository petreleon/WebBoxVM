use super::super::super::completion::{PendingCompletion, WritableRegion};
use super::super::super::protocol::*;
use super::super::super::VirtioGpu;
use super::super::virgl_draw_fixture::*;
use super::super::{full_scanout, header};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const USED: u64 = RAM_BASE + 0x1000;
const RESPONSE: u64 = RAM_BASE + 0x2000;

#[test]
fn unref_recreate_before_late_clear_preserves_new_allocation() {
    let (mut gpu, mut mem) = prepared();
    let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &create(TARGET, 2, 1, 2, 1024, 768), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(TARGET), RESP_OK_NODATA);
    gpu.resources.get_mut(&TARGET).unwrap().pixels.fill(37);

    assert!(gpu.complete_3d(&mut mem, sequence, true));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_OK_NODATA as u64));
    assert!(gpu.resources[&TARGET].pixels.iter().all(|byte| *byte == 37));
    assert!(gpu.take_scanout_update().is_empty());
    assert!(!gpu.complete_3d(&mut mem, sequence, true));
}

#[test]
fn retired_resource_remains_charged_until_its_last_job_finishes() {
    let (mut gpu, mut mem) = prepared();
    let bytes = gpu.resources[&TARGET].pixels.len();
    let total = gpu.allocated_resource_bytes;
    let count = gpu.resource_count();
    let first = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    let second = enqueue_clear(&mut gpu, &mut mem, [0.0, 1.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert!(!gpu.resources.contains_key(&TARGET));
    assert_eq!(gpu.allocated_resource_bytes, total);
    assert_eq!(gpu.resource_count(), count);
    assert!(!gpu.complete_3d(&mut mem, second, true));
    assert!(gpu.complete_3d(&mut mem, first, true));
    assert_eq!(gpu.allocated_resource_bytes, total);
    assert!(gpu.complete_3d(&mut mem, second, true));
    assert_eq!(gpu.allocated_resource_bytes, total - bytes);
    assert_eq!(gpu.resource_count(), count - 1);
}

#[test]
fn successive_retired_generations_of_one_id_are_distinct() {
    let (mut gpu, mut mem) = prepared();
    let bytes = gpu.resources[&TARGET].pixels.len();
    let base = gpu.allocated_resource_bytes - bytes;
    let first = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &create(TARGET, 2, 1, 2, 1024, 768), RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    push_u32(&mut attach, TARGET); push_u32(&mut attach, 0);
    assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &full_scanout(TARGET), RESP_OK_NODATA);
    let second = enqueue_clear(&mut gpu, &mut mem, [0.0, 1.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &create(TARGET, 2, 1, 2, 8, 8), RESP_OK_NODATA);
    assert_eq!(gpu.resource_lifetimes.retired.len(), 2);
    assert!(gpu.complete_3d(&mut mem, first, true));
    assert_eq!(gpu.allocated_resource_bytes, base + bytes + 8 * 8 * 4);
    assert_eq!(gpu.resource_lifetimes.retired.len(), 1);
    assert!(gpu.complete_3d(&mut mem, second, true));
    assert_eq!(gpu.allocated_resource_bytes, base + 8 * 8 * 4);
    assert!(gpu.resources[&TARGET].pixels.iter().all(|byte| *byte == 0));
}

#[test]
fn failed_or_cancelled_job_releases_its_retired_allocation() {
    for cancel in [false, true] {
        let (mut gpu, mut mem) = prepared();
        let bytes = gpu.resources[&TARGET].pixels.len();
        let total = gpu.allocated_resource_bytes;
        let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
        unref(&mut gpu, &mut mem);
        if cancel { gpu.cancel_3d(sequence); } else {
            assert!(gpu.complete_3d(&mut mem, sequence, false));
            assert_eq!(mem.read(RESPONSE, 4), Some(RESP_ERR_UNSPEC as u64));
        }
        assert_eq!(gpu.allocated_resource_bytes, total - bytes);
        assert!(gpu.resource_lifetimes.retired.is_empty());
        assert!(!gpu.complete_3d(&mut mem, sequence, true));
    }
}

#[test]
fn resident_completion_of_retired_target_releases_browser_object() {
    let (mut gpu, mut mem) = prepared();
    let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &create(TARGET, 2, 1, 2, 1024, 768), RESP_OK_NODATA);
    assert!(gpu.complete_3d_resident(&mut mem, sequence));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_OK_NODATA as u64));
    assert!(gpu.resources[&TARGET].pixels.iter().all(|byte| *byte == 0));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    let release = gpu.take_3d_update();
    assert_eq!(release.get(..4), Some(&b"VGL1"[..]));
    assert_eq!(read_u32(&release, 8), Some(sequence));
    assert!(gpu.resource_lifetimes.retired.is_empty());
}

#[test]
fn destroyed_context_still_invalidates_retired_work_and_reclaims_bytes() {
    let (mut gpu, mut mem) = prepared();
    let bytes = gpu.resources[&TARGET].pixels.len();
    let total = gpu.allocated_resource_bytes;
    let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
    assert!(gpu.complete_3d(&mut mem, sequence, true));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_ERR_UNSPEC as u64));
    assert_eq!(gpu.allocated_resource_bytes, total - bytes);
}

#[test]
fn reset_discards_retired_allocations_and_stale_completions() {
    let (mut gpu, mut mem) = prepared();
    let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    gpu.cold_reset(&mut mem);
    assert_eq!(gpu.allocated_resource_bytes, 0);
    assert_eq!(gpu.resource_count(), 0);
    assert!(gpu.resource_lifetimes.retired.is_empty());
    assert!(!gpu.complete_3d(&mut mem, sequence, true));
}

#[test]
fn retired_bytes_cannot_be_reused_to_exceed_the_allocation_budget() {
    let (mut gpu, mut mem) = prepared();
    let sequence = enqueue_clear(&mut gpu, &mut mem, [1.0, 0.0, 0.0, 1.0]);
    unref(&mut gpu, &mut mem);
    unref_id(&mut gpu, &mut mem, BUFFER);
    unref_id(&mut gpu, &mut mem, TEXTURE);
    assert_response(&mut gpu, &mut mem, &create(40, 2, 1, 2, 4096, 4096), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &create(41, 2, 1, 2, 4096, 4096), RESP_ERR_OUT_OF_MEMORY);
    assert!(!gpu.resources.contains_key(&41));
    assert!(gpu.complete_3d(&mut mem, sequence, true));
    assert_response(&mut gpu, &mut mem, &create(41, 2, 1, 2, 4096, 4096), RESP_OK_NODATA);
    assert_eq!(gpu.allocated_resource_bytes, super::super::super::MAX_TOTAL_RESOURCE_BYTES);
}

fn enqueue_clear(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, color: [f32; 4]) -> u32 {
    if gpu.virgl_contexts[&7].framebuffer_resource() != Some(TARGET) {
        assert_response(gpu, mem, &submit(&surface_create(9, TARGET)), RESP_OK_NODATA);
        assert_response(gpu, mem, &submit(&framebuffer(9)), RESP_OK_NODATA);
    }
    let pending = gpu.execute_queued_command(mem, &submit(&clear(color))).deferred.unwrap();
    assert!(gpu.attach_3d_completion(pending.sequence, PendingCompletion {
        header: pending.header,
        output: vec![WritableRegion { addr: RESPONSE, len: 24 }],
        used: USED, queue_size: 8, head: 0,
    }));
    assert!(!gpu.take_3d_update().is_empty());
    pending.sequence
}

fn unref(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
    unref_id(gpu, mem, TARGET);
}

fn unref_id(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, id: u32) {
    let mut command = header(CMD_RESOURCE_UNREF);
    push_u32(&mut command, id);
    push_u32(&mut command, 0);
    assert_response(gpu, mem, &command, RESP_OK_NODATA);
}
