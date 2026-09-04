use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::feature::STATUS_FEATURES_OK;
use super::super::fence::{FLAG_FENCE, FLAG_INFO_RING_IDX};
use super::super::protocol::*;
use super::super::three_d::DeferredSubmit;
use super::super::{VIRTIO_F_VERSION_1, VIRTIO_GPU_F_CONTEXT_INIT, VirtioGpu};
use super::{context_create, header, response_type, submit_3d, wbg3_packet};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const USED: u64 = RAM_BASE + 0x1000;
const FIRST_RESPONSE: u64 = RAM_BASE + 0x2000;
const SECOND_RESPONSE: u64 = RAM_BASE + 0x3000;

#[test]
fn fence_flags_require_context_init_and_canonical_ring_padding() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let mut command = fenced(header(CMD_GET_DISPLAY_INFO), 3, 41);
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);

    enable_context_init(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &command, RESP_OK_DISPLAY_INFO);
    command[4..8].copy_from_slice(&FLAG_INFO_RING_IDX.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
    command[4..8].copy_from_slice(&(FLAG_FENCE | 4).to_le_bytes());
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
    command[4..8].copy_from_slice(&FLAG_FENCE.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
    command[20..24].copy_from_slice(&0x100u32.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &command, RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn same_timeline_acknowledgments_are_ordered_and_echo_fence_headers() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    enable_context_init(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let first = defer(&mut gpu, &mut mem, fenced_submit(3, 41));
    let second = defer(&mut gpu, &mut mem, fenced_submit(3, 42));
    attach(&mut gpu, first, FIRST_RESPONSE, 1);
    attach(&mut gpu, second, SECOND_RESPONSE, 2);
    assert_eq!(read_u32(&gpu.take_3d_update(), 12), Some(first.sequence));
    assert_eq!(read_u32(&gpu.take_3d_update(), 12), Some(second.sequence));

    assert!(!gpu.complete_3d(&mut mem, second.sequence, true));
    assert_eq!(mem.read(USED + 2, 2), Some(0));
    assert!(gpu.complete_3d(&mut mem, first.sequence, true));
    assert!(gpu.complete_3d(&mut mem, second.sequence, true));
    assert_eq!(mem.read(USED + 2, 2), Some(2));
    assert_fence_response(&mem, FIRST_RESPONSE, 3, 41);
    assert_fence_response(&mem, SECOND_RESPONSE, 3, 42);
}

#[test]
fn independent_ring_timelines_can_settle_independently() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    enable_context_init(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let first = defer(&mut gpu, &mut mem, fenced_submit(1, 51));
    let second = defer(&mut gpu, &mut mem, fenced_submit(2, 52));
    attach(&mut gpu, first, FIRST_RESPONSE, 1);
    attach(&mut gpu, second, SECOND_RESPONSE, 2);
    let _ = gpu.take_3d_update();
    let _ = gpu.take_3d_update();

    assert!(gpu.complete_3d(&mut mem, second.sequence, true));
    assert!(gpu.complete_3d(&mut mem, first.sequence, true));
    assert_fence_response(&mem, FIRST_RESPONSE, 1, 51);
    assert_fence_response(&mem, SECOND_RESPONSE, 2, 52);
}

fn enable_context_init(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
    gpu.write(mem, 0x024, 0, 4);
    gpu.write(mem, 0x020, VIRTIO_GPU_F_CONTEXT_INIT, 4);
    gpu.write(mem, 0x024, 1, 4);
    gpu.write(mem, 0x020, VIRTIO_F_VERSION_1 >> 32, 4);
    gpu.write(mem, 0x070, STATUS_FEATURES_OK.into(), 4);
}

fn fenced(mut command: Vec<u8>, ring: u8, fence_id: u64) -> Vec<u8> {
    command[4..8].copy_from_slice(&(FLAG_FENCE | FLAG_INFO_RING_IDX).to_le_bytes());
    command[8..16].copy_from_slice(&fence_id.to_le_bytes());
    command[20] = ring;
    command
}

fn fenced_submit(ring: u8, fence_id: u64) -> Vec<u8> {
    fenced(submit_3d(&wbg3_packet(3, 3)), ring, fence_id)
}

fn defer(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: Vec<u8>) -> DeferredSubmit {
    gpu.execute_queued_command(mem, &command)
        .deferred
        .expect("valid submission must defer")
}

fn attach(gpu: &mut VirtioGpu, submit: DeferredSubmit, output: u64, head: u16) {
    assert!(gpu.attach_3d_completion(
        submit.sequence,
        PendingCompletion {
            header: submit.header,
            output: vec![WritableRegion { addr: output, len: 24 }],
            used: USED,
            queue_size: 8,
            head,
        },
    ));
}

fn assert_fence_response(mem: &PhysicalMemory, response: u64, ring: u8, fence_id: u64) {
    assert_eq!(mem.read(response, 4), Some(RESP_OK_NODATA as u64));
    assert_eq!(mem.read(response + 4, 4), Some((FLAG_FENCE | FLAG_INFO_RING_IDX) as u64));
    assert_eq!(mem.read(response + 8, 8), Some(fence_id));
    assert_eq!(mem.read(response + 20, 4), Some(u64::from(ring)));
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
