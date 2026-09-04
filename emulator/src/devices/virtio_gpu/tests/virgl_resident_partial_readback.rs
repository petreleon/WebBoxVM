use super::super::VirtioGpu;
use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::three_d::ResidentResource;
use super::virgl_readback::{SECOND_BACKING, RESOURCE_ID, WIDTH, assert_response, read, resource_with_backing, transfer, virgl_context};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

#[test]
fn partial_resident_transfer_reads_only_its_gpu_rect_and_keeps_gpu_authority() {
    let (mut gpu, mut mem) = prepared_resident();
    let shadow = gpu.resources[&RESOURCE_ID].pixels.clone();
    gpu.pending_damage = Some(Rect { x: 3, y: 2, width: 1, height: 1 });
    let (sequence, packet) = defer(&mut gpu, &mut mem, transfer(1, 0, 2, 2, 4));
    assert_eq!(packet.len(), 40);
    assert_eq!([4, 8, 12, 16, 20, 24, 28, 32, 36].map(|offset| read_u32(&packet, offset)),
        [Some(2), Some(sequence), Some(71), Some(WIDTH), Some(3), Some(1), Some(0), Some(2), Some(2)]);
    let rgba = [3, 2, 1, 4, 7, 6, 5, 8, 11, 10, 9, 12, 15, 14, 13, 16];
    assert!(gpu.complete_3d_readback(&mut mem, sequence, 2, &rgba));
    assert_eq!(read_u32(&read(&mem, RAM_BASE + 0x7000, 4), 0), Some(RESP_OK_NODATA));
    assert!(gpu.resident_resources.contains_key(&RESOURCE_ID));
    assert_eq!(gpu.resident_resources[&RESOURCE_ID].producer_sequence, 71);
    assert_eq!(gpu.resources[&RESOURCE_ID].pixels, shadow);
    assert_eq!(gpu.pending_damage, Some(Rect { x: 3, y: 2, width: 1, height: 1 }));
    assert!(gpu.take_3d_update().is_empty());
    assert_eq!(read(&mem, RAM_BASE + 4, 8), [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(read(&mem, RAM_BASE + 20, 4), [9, 10, 11, 12]);
    assert_eq!(read(&mem, SECOND_BACKING, 4), [13, 14, 15, 16]);
}

#[test]
fn failed_partial_resident_readback_preserves_backing_and_gpu_authority() {
    let (mut gpu, mut mem) = prepared_resident();
    let shadow = gpu.resources[&RESOURCE_ID].pixels.clone();
    let (sequence, _) = defer(&mut gpu, &mut mem, transfer(0, 0, 1, 1, 47));
    assert!(gpu.complete_3d_readback(&mut mem, sequence, 1, &[9; 4]));
    assert_eq!(read_u32(&read(&mem, RAM_BASE + 0x7000, 4), 0), Some(RESP_ERR_UNSPEC));
    assert!(gpu.resident_resources.contains_key(&RESOURCE_ID));
    assert_eq!(gpu.resident_resources[&RESOURCE_ID].producer_sequence, 71);
    assert_eq!(gpu.resources[&RESOURCE_ID].pixels, shadow);
    assert_eq!(read(&mem, RAM_BASE, 4), [0; 4]);
    assert_eq!(read(&mem, SECOND_BACKING + 23, 1), [0]);
}

fn prepared_resident() -> (VirtioGpu, PhysicalMemory) {
    let (mut gpu, mut mem) = resource_with_backing();
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    let generation = gpu.virgl_contexts[&7].generation;
    gpu.resident_resources.insert(RESOURCE_ID, ResidentResource { context_id: 7, generation, producer_sequence: 71 });
    (gpu, mem)
}

fn defer(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: Vec<u8>) -> (u32, Vec<u8>) {
    let deferred = gpu.execute_queued_command(mem, &command).deferred.expect("resident transfer defers");
    assert!(gpu.attach_3d_completion(deferred.sequence, PendingCompletion {
        header: deferred.header, output: vec![WritableRegion { addr: RAM_BASE + 0x7000, len: 24 }],
        used: RAM_BASE + 0x7100, queue_size: 8, head: 1,
    }));
    (deferred.sequence, gpu.take_3d_update())
}
