use super::super::super::protocol::*;
use super::super::super::three_d::ResidentResource;
use super::super::super::VirtioGpu;
use super::super::virgl_draw_fixture::{create, submit};
use super::super::virgl_readback::{assert_response, resource_with_backing, virgl_context, RESOURCE_ID};
use super::super::header;
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;
use super::super::super::completion::{PendingCompletion, WritableRegion};

#[test]
fn full_release_queue_rejects_unref_without_mutating_allocation_or_owner() {
    let (mut gpu, mut mem) = prepared();
    gpu.resident_releases.extend(100..116);
    let bytes = gpu.allocated_resource_bytes;
    let epoch = gpu.resident_epoch;
    assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, RESOURCE_ID), RESP_ERR_OUT_OF_MEMORY);
    assert_eq!(gpu.allocated_resource_bytes, bytes);
    assert_eq!(gpu.resident_epoch, epoch);
    assert_eq!(gpu.resident_resources[&RESOURCE_ID].producer_sequence, 71);
    assert!(gpu.resources.contains_key(&RESOURCE_ID));
    assert!(gpu.is_virgl_resource(RESOURCE_ID));
    assert_eq!(gpu.resident_releases.len(), 16);
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(100));
    assert_response(&mut gpu, &mut mem, &resource(CMD_RESOURCE_UNREF, RESOURCE_ID), RESP_OK_NODATA);
    assert_eq!(drain(&mut gpu), (101..116).chain([71]).collect::<Vec<_>>());
}

#[test]
fn context_destroy_reserves_all_releases_before_changing_context() {
    let (mut gpu, mut mem) = prepared();
    assert_response(&mut gpu, &mut mem, &create(2, 2, 1, 2, 4, 3), RESP_OK_NODATA);
    let owner = gpu.resident_resources[&RESOURCE_ID];
    gpu.resident_resources.insert(2, ResidentResource { producer_sequence: 72, ..owner });
    gpu.resident_releases.extend(100..115);
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_ERR_OUT_OF_MEMORY);
    assert!(gpu.contexts.contains_key(&7));
    assert_eq!(gpu.virgl_contexts[&7].generation, owner.generation);
    assert_eq!(gpu.resident_resources.len(), 2);
    assert_eq!(gpu.resident_releases.len(), 15);
    gpu.take_3d_update();
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
    assert!(!gpu.contexts.contains_key(&7));
    let mut released = drain(&mut gpu); released.sort_unstable();
    assert_eq!(released, [71, 72].into_iter().chain(101..115).collect::<Vec<_>>());
}

#[test]
fn full_release_queue_rejects_cpu_uploads_and_copy_before_pixel_mutation() {
    for command_type in [CMD_TRANSFER_TO_HOST_2D, CMD_TRANSFER_TO_HOST_3D, CMD_SUBMIT_3D] {
        let (mut gpu, mut mem) = prepared();
        assert_response(&mut gpu, &mut mem, &create(2, 2, 1, 2, 4, 3), RESP_OK_NODATA);
        for id in [1, 2] { assert_response(&mut gpu, &mut mem, &resource(CMD_CTX_ATTACH_RESOURCE, id), RESP_OK_NODATA); }
        gpu.resources.get_mut(&2).unwrap().pixels.fill(9);
        mem.write_bytes(RAM_BASE, &[9; 24]).unwrap();
        mem.write_bytes(RAM_BASE + 0x100, &[9; 24]).unwrap();
        gpu.resident_releases.extend(100..116);
        let command = match command_type {
            CMD_TRANSFER_TO_HOST_2D => upload_2d(),
            CMD_TRANSFER_TO_HOST_3D => upload_3d(),
            _ => submit(&[17 | (13 << 16), 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 4, 3, 1]),
        };
        assert_response(&mut gpu, &mut mem, &command, RESP_ERR_OUT_OF_MEMORY);
        assert!(gpu.resources[&1].pixels.iter().all(|byte| *byte == 0));
        assert_eq!(gpu.resident_resources[&1].producer_sequence, 71);
        assert_eq!(gpu.resident_releases.len(), 16);
        gpu.take_3d_update();
        assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);
        assert!(gpu.resources[&1].pixels.iter().all(|byte| *byte == 9));
        assert_eq!(drain(&mut gpu), (101..116).chain([71]).collect::<Vec<_>>());
    }
}

#[test]
fn acknowledged_cpu_and_gpu_overwrites_keep_release_delivery_bounded() {
    use super::super::virgl_draw_fixture::*;
    use super::super::virgl_solid_batch::{configure, constants};
    for readback in [false, true] {
        let (mut gpu, mut mem) = super::super::virgl_draw_fixture::prepared();
        gpu.resident_resources.insert(TARGET, ResidentResource {
            context_id: 7, generation: gpu.virgl_contexts[&7].generation, producer_sequence: 71,
        });
        let mut words = clear([1.0, 0.0, 0.0, 1.0]);
        if readback {
            configure(&mut gpu, &mut mem);
            words.extend(constants([1.0, 0.0, 0.0, 0.5])); words.extend(draw());
            words.extend(constants([0.0, 1.0, 0.0, 0.5])); words.extend(draw());
        } else {
            let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
            assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
        }
        let pending = gpu.execute_queued_command(&mut mem, &submit(&words)).deferred.unwrap();
        assert!(gpu.attach_3d_completion(pending.sequence, PendingCompletion {
            header: pending.header, output: vec![WritableRegion { addr: RAM_BASE + 0x2000, len: 24 }],
            used: RAM_BASE + 0x1000, queue_size: 8, head: 0,
        }));
        assert!(!gpu.take_3d_update().is_empty());
        gpu.resident_releases.extend(100..116);
        if readback {
            assert!(gpu.complete_3d_readback(&mut mem, pending.sequence, 1, &[9; 4].repeat(1024 * 768)));
        } else { assert!(gpu.complete_3d(&mut mem, pending.sequence, true)); }
        assert_eq!(mem.read(RAM_BASE + 0x2000, 4), Some(RESP_OK_NODATA as u64));
        assert_eq!(gpu.pending_3d.len(), 1);
        assert_eq!(gpu.pending_3d_bytes, 12);
        assert_eq!(gpu.resident_releases.len(), 16);
        assert!(!gpu.complete_3d(&mut mem, pending.sequence, true));
        let mut released = drain(&mut gpu); released.sort_unstable();
        assert_eq!(released, [71].into_iter().chain(100..116).collect::<Vec<_>>());
        assert_eq!(gpu.pending_3d_bytes, 0);
    }
}

fn prepared() -> (VirtioGpu, PhysicalMemory) {
    let (mut gpu, mut mem) = resource_with_backing();
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    gpu.resident_resources.insert(RESOURCE_ID, ResidentResource {
        context_id: 7, generation: gpu.virgl_contexts[&7].generation, producer_sequence: 71,
    });
    (gpu, mem)
}

pub(super) fn resource(command: u32, id: u32) -> Vec<u8> {
    let mut bytes = header(command); push_u32(&mut bytes, id); push_u32(&mut bytes, 0); bytes
}

fn upload_2d() -> Vec<u8> {
    let mut command = header(CMD_TRANSFER_TO_HOST_2D);
    for value in [0, 0, 4, 3] { push_u32(&mut command, value); }
    push_u64(&mut command, 0); push_u32(&mut command, 1); push_u32(&mut command, 0); command
}

fn upload_3d() -> Vec<u8> {
    let mut command = super::super::virgl_readback::transfer(0, 0, 4, 3, 0);
    command[..4].copy_from_slice(&CMD_TRANSFER_TO_HOST_3D.to_le_bytes()); command
}

pub(super) fn drain(gpu: &mut VirtioGpu) -> Vec<u32> {
    let mut sequences = Vec::new();
    loop {
        let packet = gpu.take_3d_update();
        if packet.is_empty() { return sequences; }
        assert_eq!(packet.get(..4), Some(&b"VGL1"[..]));
        sequences.push(read_u32(&packet, 8).unwrap());
    }
}
