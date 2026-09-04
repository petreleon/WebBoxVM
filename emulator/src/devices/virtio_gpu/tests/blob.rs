use super::super::feature::STATUS_FEATURES_OK;
use super::super::protocol::*;
use super::super::{
    VIRTIO_F_VERSION_1, VIRTIO_GPU_F_RESOURCE_BLOB, VirtioGpu,
};
use super::{create_2d, header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const BLOB_MEM_GUEST: u32 = 1;

#[test]
fn guest_blob_requires_feature_acceptance_and_releases_its_budget() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let request = blob(9, BLOB_MEM_GUEST, 0, 0, 256, &[(RAM_BASE, 4096)]);
    assert_response(&mut gpu, &mut mem, &request, RESP_ERR_UNSPEC);
    assert!(gpu.blobs.is_empty());

    negotiate_blob(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &request, RESP_OK_NODATA);
    assert_eq!(gpu.blobs[&9].size, 256);
    assert_eq!(gpu.blobs[&9].backing.len(), 1);
    assert_eq!(gpu.allocated_resource_bytes, 256);
    assert_response(&mut gpu, &mut mem, &unref(9), RESP_OK_NODATA);
    assert!(gpu.blobs.is_empty());
    assert_eq!(gpu.allocated_resource_bytes, 0);
}

#[test]
fn blob_ids_share_the_normal_resource_namespace() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_blob(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(4, BLOB_MEM_GUEST, 0, 0, 128, &[(RAM_BASE, 4096)]),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &create_2d(4, 1, 1, 1),
        RESP_ERR_INVALID_RESOURCE_ID,
    );
}

#[test]
fn guest_blob_can_attach_and_detach_backing_after_creation() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_blob(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(5, BLOB_MEM_GUEST, 0, 0, 128, &[]),
        RESP_OK_NODATA,
    );
    assert!(gpu.blobs[&5].backing.is_empty());
    assert_response(&mut gpu, &mut mem, &attach(5, 4096), RESP_OK_NODATA);
    assert_eq!(gpu.blobs[&5].backing.len(), 1);
    assert_response(&mut gpu, &mut mem, &detach(5), RESP_OK_NODATA);
    assert!(gpu.blobs[&5].backing.is_empty());
}

#[test]
fn guest_blob_can_attach_to_a_virgl_context_without_becoming_drawable() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_blob(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(6, BLOB_MEM_GUEST, 0, 0, 128, &[(RAM_BASE, 4096)]),
        RESP_OK_NODATA,
    );
    assert_response(&mut gpu, &mut mem, &context_resource(CMD_CTX_ATTACH_RESOURCE, 6), RESP_OK_NODATA);
    assert!(gpu.virgl_contexts[&7].is_attached(6));
    assert!(!gpu.is_virgl_resource(6));
    assert_response(&mut gpu, &mut mem, &context_resource(CMD_CTX_DETACH_RESOURCE, 6), RESP_OK_NODATA);
}

#[test]
fn malformed_or_unsupported_blob_shapes_leave_state_unchanged() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_blob(&mut gpu, &mut mem);
    let cases = [
        blob(1, 2, 0, 0, 64, &[(RAM_BASE, 4096)]),
        blob(1, BLOB_MEM_GUEST, 1, 0, 64, &[(RAM_BASE, 4096)]),
        blob(1, BLOB_MEM_GUEST, 0, 7, 64, &[(RAM_BASE, 4096)]),
        blob(1, BLOB_MEM_GUEST, 0, 0, 0, &[(RAM_BASE, 4096)]),
        blob(1, BLOB_MEM_GUEST, 0, 0, 4097, &[(RAM_BASE, 4096)]),
        blob(1, BLOB_MEM_GUEST, 0, 0, 64, &[(RAM_BASE - 1, 4096)]),
    ];
    for request in cases {
        assert_response(&mut gpu, &mut mem, &request, RESP_ERR_INVALID_PARAMETER);
        assert!(gpu.blobs.is_empty());
        assert_eq!(gpu.allocated_resource_bytes, 0);
    }
    let mut trailing = blob(1, BLOB_MEM_GUEST, 0, 0, 64, &[(RAM_BASE, 4096)]);
    trailing.push(0);
    assert_response(&mut gpu, &mut mem, &trailing, RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.blobs.is_empty());
}

fn negotiate_blob(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
    gpu.write(mem, 0x024, 0, 4);
    gpu.write(mem, 0x020, VIRTIO_GPU_F_RESOURCE_BLOB, 4);
    gpu.write(mem, 0x024, 1, 4);
    gpu.write(mem, 0x020, VIRTIO_F_VERSION_1 >> 32, 4);
    gpu.write(mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(STATUS_FEATURES_OK.into()));
}

fn blob(id: u32, memory: u32, flags: u32, blob_id: u64, size: u64, entries: &[(u64, u32)]) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_CREATE_BLOB);
    push_u32(&mut request, id);
    push_u32(&mut request, memory);
    push_u32(&mut request, flags);
    push_u32(&mut request, entries.len() as u32);
    push_u64(&mut request, blob_id);
    push_u64(&mut request, size);
    for &(addr, len) in entries {
        push_u64(&mut request, addr);
        push_u32(&mut request, len);
        push_u32(&mut request, 0);
    }
    request
}

fn unref(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_UNREF);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn virgl_context() -> Vec<u8> {
    let mut request = header(CMD_CTX_CREATE);
    push_u32(&mut request, 4);
    push_u32(&mut request, 1);
    request.extend_from_slice(b"blob");
    request.resize(96, 0);
    request
}

fn context_resource(command: u32, id: u32) -> Vec<u8> {
    let mut request = header(command);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn attach(id: u32, len: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut request, id);
    push_u32(&mut request, 1);
    push_u64(&mut request, RAM_BASE);
    push_u32(&mut request, len);
    push_u32(&mut request, 0);
    request
}

fn detach(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_DETACH_BACKING);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, request: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, request)), expected);
}
