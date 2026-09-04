use super::super::feature::STATUS_FEATURES_OK;
use super::super::protocol::*;
use super::super::{
    VIRTIO_F_VERSION_1, VIRTIO_GPU_F_BLOB_ALIGNMENT, VIRTIO_GPU_F_CONTEXT_INIT,
    VIRTIO_GPU_F_RESOURCE_BLOB, VIRTIO_GPU_F_VIRGL, VirtioGpu,
};
use super::{header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const BLOB_MEM_HOST3D_GUEST: u32 = 3;
const BLOB_BYTES: u64 = 4096;
const RESOURCE_ID: u32 = 37;

#[test]
fn default_blob_transfers_guest_shadow_data_in_both_directions() {
    let (mut gpu, mut mem) = prepared_blob(&[(RAM_BASE, BLOB_BYTES as u32)]);
    let guest_offset = 128;
    let host_offset = 64;
    let source = [3, 1, 4, 1];
    mem.write_bytes(RAM_BASE + guest_offset, &source).unwrap();
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_TO_HOST_3D, host_offset, guest_offset, source.len() as u32),
        RESP_OK_NODATA,
    );
    let host = gpu.blobs[&RESOURCE_ID].host.as_ref().unwrap();
    assert_eq!(&host.bytes[host_offset as usize..host_offset as usize + source.len()], &source);

    let expected = [2, 7, 1, 8];
    gpu.blobs
        .get_mut(&RESOURCE_ID)
        .unwrap()
        .host
        .as_mut()
        .unwrap()
        .bytes[host_offset as usize..host_offset as usize + expected.len()]
        .copy_from_slice(&expected);
    assert_response(
        &mut gpu,
        &mut mem,
        &transfer(CMD_TRANSFER_FROM_HOST_3D, host_offset, guest_offset, expected.len() as u32),
        RESP_OK_NODATA,
    );
    let mut restored = [0; 4];
    mem.read_bytes(RAM_BASE + guest_offset, &mut restored).unwrap();
    assert_eq!(restored, expected);
    assert_response(&mut gpu, &mut mem, &map(RESOURCE_ID), RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn default_blob_supports_late_backing_and_refuses_detached_shadow_transfers() {
    let (mut gpu, mut mem) = prepared_blob(&[]);
    let transfer = transfer(CMD_TRANSFER_TO_HOST_3D, 0, 0, 4);
    assert_response(&mut gpu, &mut mem, &transfer, RESP_ERR_INVALID_PARAMETER);
    assert_response(&mut gpu, &mut mem, &attach(RESOURCE_ID), RESP_OK_NODATA);
    mem.write_bytes(RAM_BASE, &[5, 6, 7, 8]).unwrap();
    assert_response(&mut gpu, &mut mem, &transfer, RESP_OK_NODATA);
    assert_eq!(&gpu.blobs[&RESOURCE_ID].host.as_ref().unwrap().bytes[..4], &[5, 6, 7, 8]);
    assert_response(&mut gpu, &mut mem, &detach(RESOURCE_ID), RESP_OK_NODATA);
    mem.write_bytes(RAM_BASE, &[9, 9, 9, 9]).unwrap();
    assert_response(&mut gpu, &mut mem, &transfer, RESP_ERR_INVALID_PARAMETER);
    assert_eq!(&gpu.blobs[&RESOURCE_ID].host.as_ref().unwrap().bytes[..4], &[5, 6, 7, 8]);
}

fn prepared_blob(entries: &[(u64, u32)]) -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(RESOURCE_ID, entries),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, &context_resource(CMD_CTX_ATTACH_RESOURCE), RESP_OK_NODATA);
    (gpu, mem)
}

fn negotiate(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
    gpu.write(mem, 0x024, 0, 4);
    gpu.write(
        mem,
        0x020,
        VIRTIO_GPU_F_VIRGL
            | VIRTIO_GPU_F_RESOURCE_BLOB
            | VIRTIO_GPU_F_CONTEXT_INIT
            | VIRTIO_GPU_F_BLOB_ALIGNMENT,
        4,
    );
    gpu.write(mem, 0x024, 1, 4);
    gpu.write(mem, 0x020, VIRTIO_F_VERSION_1 >> 32, 4);
    gpu.write(mem, 0x070, STATUS_FEATURES_OK.into(), 4);
}

fn virgl_context() -> Vec<u8> {
    let mut request = header(CMD_CTX_CREATE);
    push_u32(&mut request, 4);
    push_u32(&mut request, 1);
    request.extend_from_slice(b"blob");
    request.resize(96, 0);
    request
}

fn blob(id: u32, entries: &[(u64, u32)]) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_CREATE_BLOB);
    for value in [id, BLOB_MEM_HOST3D_GUEST, 0, entries.len() as u32] {
        push_u32(&mut request, value);
    }
    push_u64(&mut request, 0);
    push_u64(&mut request, BLOB_BYTES);
    for &(addr, len) in entries {
        push_u64(&mut request, addr);
        push_u32(&mut request, len);
        push_u32(&mut request, 0);
    }
    request
}

fn context_resource(command: u32) -> Vec<u8> {
    let mut request = header(command);
    push_u32(&mut request, RESOURCE_ID);
    push_u32(&mut request, 0);
    request
}

fn transfer(command: u32, x: u32, offset: u64, width: u32) -> Vec<u8> {
    let mut request = header(command);
    for value in [x, 0, 0, width, 1, 1] {
        push_u32(&mut request, value);
    }
    push_u64(&mut request, offset);
    for value in [RESOURCE_ID, 0, 0, 0] {
        push_u32(&mut request, value);
    }
    request
}

fn attach(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut request, id);
    push_u32(&mut request, 1);
    push_u64(&mut request, RAM_BASE);
    push_u32(&mut request, BLOB_BYTES as u32);
    push_u32(&mut request, 0);
    request
}

fn detach(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_DETACH_BACKING);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn map(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_MAP_BLOB);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    push_u64(&mut request, 0);
    request
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, request: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, request)), expected);
}
