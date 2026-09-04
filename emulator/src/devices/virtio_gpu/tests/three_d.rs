use super::super::protocol::*;
use super::super::three_d::packet::MAX_WBG3_PACKET_BYTES;
use super::super::three_d::{
    CAPSET_ID, CAPSET_SIZE, CAPSET_VERSION, MAX_3D_INDICES, MAX_3D_VERTICES,
};
use super::super::{
    VIRTIO_GPU_F_BLOB_ALIGNMENT, VIRTIO_GPU_F_CONTEXT_INIT, VIRTIO_GPU_F_RESOURCE_BLOB,
    VIRTIO_GPU_F_VIRGL, VirtioGpu,
};
use super::{context_create, header, response_type, submit_3d, wbg3_packet};
use crate::memory::PhysicalMemory;

#[test]
fn features_and_private_capset_remain_advertised_with_exact_structures() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_eq!(
        gpu.read(0x010, 4),
        Some(
            VIRTIO_GPU_F_VIRGL
                | VIRTIO_GPU_F_RESOURCE_BLOB
                | VIRTIO_GPU_F_CONTEXT_INIT
                | VIRTIO_GPU_F_BLOB_ALIGNMENT
        )
    );
    gpu.write(&mut mem, 0x014, 1, 4);
    assert_eq!(gpu.read(0x010, 4), Some(1));
    assert_eq!(gpu.read(0x10c, 4), Some(2));

    let mut info = header(CMD_GET_CAPSET_INFO);
    push_u32(&mut info, 1);
    push_u32(&mut info, 0);
    let response = gpu.execute_command(&mut mem, &info);
    assert_eq!(response_type(&response), RESP_OK_CAPSET_INFO);
    assert_eq!(response.len(), 40);
    assert_eq!(read_u32(&response, 24), Some(CAPSET_ID));
    assert_eq!(read_u32(&response, 28), Some(CAPSET_VERSION));
    assert_eq!(read_u32(&response, 32), Some(CAPSET_SIZE));

    let mut get = header(CMD_GET_CAPSET);
    push_u32(&mut get, CAPSET_ID);
    push_u32(&mut get, CAPSET_VERSION);
    let response = gpu.execute_command(&mut mem, &get);
    assert_eq!(response_type(&response), RESP_OK_CAPSET);
    assert_eq!(response.len(), 24 + CAPSET_SIZE as usize);
    assert_eq!(&response[24..28], b"WBG3");
    assert_eq!(read_u32(&response, 28), Some(1));
    assert_eq!(read_u32(&response, 32), Some(8192));
    assert_eq!(read_u32(&response, 36), Some(MAX_3D_VERTICES));
    assert_eq!(read_u32(&response, 40), Some(MAX_3D_INDICES));
}

#[test]
fn private_contexts_remain_checked_and_unknown_capsets_reject() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &context_create(),
        RESP_ERR_INVALID_CONTEXT_ID,
    );
    let destroy = header(CMD_CTX_DESTROY);
    assert_response(&mut gpu, &mut mem, &destroy, RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &destroy, RESP_ERR_INVALID_CONTEXT_ID);

    let mut wrong_capset = context_create();
    wrong_capset[28..32].copy_from_slice(&6u32.to_le_bytes());
    assert_response(
        &mut gpu,
        &mut mem,
        &wrong_capset,
        RESP_ERR_INVALID_PARAMETER,
    );
    let mut oversized = context_create();
    oversized.push(0);
    assert_response(&mut gpu, &mut mem, &oversized, RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn submit_validates_wbg3_and_overwrites_guest_sequence() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let packet = wbg3_packet(3, 3);
    let result = gpu.execute_queued_command(&mut mem, &submit_3d(&packet));
    assert_eq!(response_type(&result.response), RESP_OK_NODATA);
    assert_eq!(result.deferred.map(|submit| submit.sequence), Some(1));
    let exported = gpu.take_3d_update();
    assert_eq!(read_u32(&exported, 12), Some(1));
    assert_eq!(&exported[..12], &packet[..12]);
    assert_eq!(&exported[16..], &packet[16..]);
    assert!(gpu.take_3d_update().is_empty());
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
}

#[test]
fn malformed_wbg3_payloads_are_rejected_without_queueing() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let base = wbg3_packet(3, 3);
    let mut bad_magic = base.clone();
    bad_magic[0] = b'X';
    let mut bad_version = base.clone();
    bad_version[4..8].copy_from_slice(&2u32.to_le_bytes());
    let mut bad_opcode = base.clone();
    bad_opcode[8..12].copy_from_slice(&2u32.to_le_bytes());
    let mut bad_count = base.clone();
    bad_count[28..32].copy_from_slice(&2u32.to_le_bytes());
    for packet in [bad_magic, bad_version, bad_opcode, bad_count] {
        assert_response(
            &mut gpu,
            &mut mem,
            &submit_3d(&packet),
            RESP_ERR_INVALID_PARAMETER,
        );
    }
    let mut nan = base.clone();
    nan[32..36].copy_from_slice(&f32::NAN.to_le_bytes());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit_3d(&nan),
        RESP_ERR_INVALID_PARAMETER,
    );
    let mut bad_index = base.clone();
    let end = bad_index.len();
    bad_index[end - 2..].copy_from_slice(&3u16.to_le_bytes());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit_3d(&bad_index),
        RESP_ERR_INVALID_PARAMETER,
    );
    let mut zero_width = base;
    zero_width[16..20].copy_from_slice(&0u32.to_le_bytes());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit_3d(&zero_width),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());
}

#[test]
fn maximum_packet_size_is_exact_and_pending_bytes_are_bounded() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let packet = wbg3_packet(MAX_3D_VERTICES, MAX_3D_INDICES);
    assert_eq!(packet.len(), MAX_WBG3_PACKET_BYTES);
    let command = submit_3d(&packet);
    for _ in 0..15 {
        let result = gpu.execute_queued_command(&mut mem, &command);
        assert_eq!(response_type(&result.response), RESP_OK_NODATA);
        assert!(result.deferred.is_some());
    }
    let result = gpu.execute_queued_command(&mut mem, &command);
    assert_eq!(response_type(&result.response), RESP_ERR_OUT_OF_MEMORY);
    assert!(result.deferred.is_none());
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
