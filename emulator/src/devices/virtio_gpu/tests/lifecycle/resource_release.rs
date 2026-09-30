use super::super::super::completion::{PendingCompletion, WritableRegion};
use super::super::super::protocol::*;
use super::super::super::{Scanout, VirtioGpu, MAX_PENDING_3D_SUBMITS};
use super::super::virgl_draw_fixture::*;
use super::super::header;
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;
use std::collections::BTreeSet;

#[test]
fn saturated_retired_resident_completions_deliver_every_old_and_new_release() {
    let (mut gpu, mut mem) = prepared();
    let base_bytes = gpu.allocated_resource_bytes;
    let mut expected = BTreeSet::new();
    for id in 100..116 {
        assert_response(&mut gpu, &mut mem, &create(id, 2, 1, 2, 128, 128), RESP_OK_NODATA);
        let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
        push_u32(&mut attach, id); push_u32(&mut attach, 0);
        assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
        assert_response(&mut gpu, &mut mem, &submit(&surface_create(id + 1000, id)), RESP_OK_NODATA);
        let sequence = enqueue(&mut gpu, &mut mem, id);
        expected.insert(sequence);
        assert!(gpu.complete_3d_resident(&mut mem, sequence));
    }
    assert_eq!(gpu.resident_resources.len(), 16);
    let pending: Vec<_> = (100..116).map(|id| enqueue(&mut gpu, &mut mem, id)).collect();
    expected.extend(&pending);
    for id in 100..116 {
        let mut unref = header(CMD_RESOURCE_UNREF);
        push_u32(&mut unref, id); push_u32(&mut unref, 0);
        assert_response(&mut gpu, &mut mem, &unref, RESP_OK_NODATA);
    }
    assert_eq!(gpu.resident_releases.len(), 16);
    for sequence in pending { assert!(gpu.complete_3d_resident(&mut mem, sequence)); }
    assert_eq!(gpu.allocated_resource_bytes, base_bytes);
    assert!(gpu.resource_lifetimes.retired.is_empty());
    assert_eq!(gpu.pending_3d.len(), MAX_PENDING_3D_SUBMITS);
    assert_eq!(gpu.pending_3d_bytes, 16 * 12);
    gpu.scanout = Some(Scanout { resource_id: TARGET, rect: Rect { x: 0, y: 0, width: 1024, height: 768 } });
    let mut retry = surface_create(9, TARGET);
    retry.extend(framebuffer(9)); retry.extend(clear([0.0, 1.0, 0.0, 1.0]));
    assert_response(&mut gpu, &mut mem, &submit(&retry), RESP_ERR_OUT_OF_MEMORY);
    let mut actual = BTreeSet::new();
    loop {
        assert!(gpu.pending_3d.len() <= MAX_PENDING_3D_SUBMITS);
        let packet = gpu.take_3d_update();
        if packet.is_empty() { break; }
        assert_eq!(packet.get(..4), Some(&b"VGL1"[..]));
        assert!(actual.insert(read_u32(&packet, 8).unwrap()), "duplicate release");
    }
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 32);
    assert_eq!(gpu.pending_3d_bytes, 0);
    assert!(gpu.pending_3d.is_empty());
    let admitted = gpu.execute_queued_command(&mut mem, &submit(&retry)).deferred.unwrap();
    gpu.cancel_3d(admitted.sequence);
}

fn enqueue(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, id: u32) -> u32 {
    // Exercise small internal render targets; the public fixed-size KMS contract is unchanged.
    gpu.scanout = Some(Scanout { resource_id: id, rect: Rect { x: 0, y: 0, width: 128, height: 128 } });
    let mut words = framebuffer(id + 1000);
    words.extend(clear([1.0, 0.0, 0.0, 1.0]));
    let pending = gpu.execute_queued_command(mem, &submit(&words)).deferred.unwrap();
    assert!(gpu.attach_3d_completion(pending.sequence, PendingCompletion {
        header: pending.header, output: vec![WritableRegion { addr: RAM_BASE + 0x2000, len: 24 }],
        used: RAM_BASE + 0x1000, queue_size: 64, head: 0,
    }));
    assert_eq!(gpu.take_3d_update().get(..4), Some(&b"VGC1"[..]));
    pending.sequence
}
