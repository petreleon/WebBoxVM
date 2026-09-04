use super::super::feature::STATUS_FEATURES_OK;
use super::super::protocol::*;
use super::super::{
    VIRTIO_F_VERSION_1, VIRTIO_GPU_F_BLOB_ALIGNMENT, VIRTIO_GPU_F_CONTEXT_INIT,
    VIRTIO_GPU_F_RESOURCE_BLOB, VIRTIO_GPU_F_VIRGL, VirtioGpu,
};
use super::response_type;
use crate::memory::PhysicalMemory;

const BLOB_MEM_HOST3D_GUEST: u32 = 3;
const BLOB_MEM_HOST3D: u32 = 2;
const BLOB_FLAG_USE_MAPPABLE: u32 = 1;
const BLOB_BYTES: u64 = 4096;
const BLOB_ID: u64 = 0x5742_4c4f_4341_4c01;

#[test]
fn renderer_local_blob_requires_a_matching_single_use_object() {
    let (mut gpu, mut mem) = prepared_gpu();
    assert_response(&mut gpu, &mut mem, &blob(7, 31, BLOB_ID, BLOB_BYTES), RESP_ERR_INVALID_PARAMETER);
    assert_response(&mut gpu, &mut mem, &prepare(7, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &blob(7, 31, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &blob(7, 32, BLOB_ID, BLOB_BYTES), RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn rejected_renderer_local_create_leaves_the_object_available_for_retry() {
    let (mut gpu, mut mem) = prepared_gpu();
    assert_response(&mut gpu, &mut mem, &prepare(7, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(7, 31, BLOB_ID, BLOB_BYTES * 2),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_response(&mut gpu, &mut mem, &blob(7, 31, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
}

#[test]
fn renderer_local_mappable_blob_uses_its_own_metadata() {
    let (mut gpu, mut mem) = prepared_gpu();
    assert_response(&mut gpu, &mut mem, &prepare_host(7, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &host_blob(7, 31, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &blob(7, 32, BLOB_ID, BLOB_BYTES), RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn renderer_local_objects_are_context_scoped_and_destroyed_with_the_context() {
    let (mut gpu, mut mem) = prepared_gpu();
    assert_response(&mut gpu, &mut mem, &context(8), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &prepare(7, BLOB_ID, BLOB_BYTES), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &blob(8, 31, BLOB_ID, BLOB_BYTES), RESP_ERR_INVALID_PARAMETER);
    assert_response(&mut gpu, &mut mem, &header(7, CMD_CTX_DESTROY), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &context(7), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &blob(7, 31, BLOB_ID, BLOB_BYTES), RESP_ERR_INVALID_PARAMETER);
}

fn prepared_gpu() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &context(7), RESP_OK_NODATA);
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

fn context(id: u32) -> Vec<u8> {
    let mut request = header(id, CMD_CTX_CREATE);
    push_u32(&mut request, 4);
    push_u32(&mut request, 1);
    request.extend_from_slice(b"blob");
    request.resize(96, 0);
    request
}

fn prepare(context: u32, blob_id: u64, size: u64) -> Vec<u8> {
    prepare_with(context, blob_id, size, BLOB_MEM_HOST3D_GUEST, 0)
}

fn prepare_host(context: u32, blob_id: u64, size: u64) -> Vec<u8> {
    prepare_with(context, blob_id, size, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE)
}

fn prepare_with(context: u32, blob_id: u64, size: u64, memory: u32, flags: u32) -> Vec<u8> {
    let mut request = header(context, CMD_SUBMIT_3D);
    push_u32(&mut request, 32);
    push_u32(&mut request, 0);
    request.extend_from_slice(b"WBL1");
    push_u32(&mut request, 1);
    push_u64(&mut request, blob_id);
    push_u64(&mut request, size);
    push_u32(&mut request, memory);
    push_u32(&mut request, flags);
    request
}

fn blob(context: u32, resource: u32, blob_id: u64, size: u64) -> Vec<u8> {
    blob_with(context, resource, blob_id, size, BLOB_MEM_HOST3D_GUEST, 0)
}

fn host_blob(context: u32, resource: u32, blob_id: u64, size: u64) -> Vec<u8> {
    blob_with(context, resource, blob_id, size, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE)
}

fn blob_with(context: u32, resource: u32, blob_id: u64, size: u64, memory: u32, flags: u32) -> Vec<u8> {
    let mut request = header(context, CMD_RESOURCE_CREATE_BLOB);
    for value in [resource, memory, flags, 0] {
        push_u32(&mut request, value);
    }
    push_u64(&mut request, blob_id);
    push_u64(&mut request, size);
    request
}

fn header(context: u32, command: u32) -> Vec<u8> {
    let mut request = Vec::new();
    push_u32(&mut request, command);
    push_u32(&mut request, 1);
    push_u64(&mut request, 0x1122_3344_5566_7788);
    push_u32(&mut request, context);
    push_u32(&mut request, 0);
    request
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, request: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, request)), expected);
}
