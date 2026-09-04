use super::super::feature::STATUS_FEATURES_OK;
use super::super::protocol::*;
use super::super::{
    VIRTIO_F_VERSION_1, VIRTIO_GPU_F_BLOB_ALIGNMENT, VIRTIO_GPU_F_CONTEXT_INIT,
    VIRTIO_GPU_F_RESOURCE_BLOB, VIRTIO_GPU_F_VIRGL, VirtioGpu,
};
use super::{header, response_type};
use crate::constants::{VIRTIO_GPU_HOST_VISIBLE_BASE, VIRTIO_GPU_HOST_VISIBLE_SIZE};
use crate::memory::PhysicalMemory;

const BLOB_MEM_GUEST: u32 = 1;
const BLOB_MEM_HOST3D: u32 = 2;
const BLOB_FLAG_USE_MAPPABLE: u32 = 1;

#[test]
fn mapped_host_blob_preserves_contents_and_reclaims_the_aperture() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_host_visible(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(21, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, 0, 4096, &[]),
        RESP_OK_NODATA,
    );

    let response = gpu.execute_command(&mut mem, &map(21, 0));
    assert_eq!(response_type(&response), RESP_OK_MAP_INFO);
    assert_eq!(response.len(), 32);
    assert_eq!(read_u32(&response, 24), Some(1));
    mem.write_bytes(VIRTIO_GPU_HOST_VISIBLE_BASE, &[4, 8, 15, 16])
        .unwrap();
    assert_response(&mut gpu, &mut mem, &unmap(21), RESP_OK_NODATA);
    assert_eq!(mem.read(VIRTIO_GPU_HOST_VISIBLE_BASE, 4), Some(0));

    assert_eq!(response_type(&gpu.execute_command(&mut mem, &map(21, 0))), RESP_OK_MAP_INFO);
    let mut restored = [0; 4];
    mem.read_bytes(VIRTIO_GPU_HOST_VISIBLE_BASE, &mut restored).unwrap();
    assert_eq!(restored, [4, 8, 15, 16]);
    assert_response(&mut gpu, &mut mem, &unref(21), RESP_OK_NODATA);
    assert_eq!(mem.read(VIRTIO_GPU_HOST_VISIBLE_BASE, 4), Some(0));
}

#[test]
fn mapping_rejects_guest_blobs_duplicates_and_out_of_range_offsets() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_host_visible(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(22, BLOB_MEM_GUEST, 0, 0, 4096, &[(crate::constants::RAM_BASE, 4096)]),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(23, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, 0, 4096, &[]),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(25, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, 1, 4096, &[]),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_response(&mut gpu, &mut mem, &map(22, 0), RESP_ERR_INVALID_PARAMETER);
    assert_response(&mut gpu, &mut mem, &map(23, 1), RESP_ERR_INVALID_PARAMETER);
    assert_response(
        &mut gpu,
        &mut mem,
        &map(23, VIRTIO_GPU_HOST_VISIBLE_SIZE),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(response_type(&gpu.execute_command(&mut mem, &map(23, 0))), RESP_OK_MAP_INFO);
    assert_response(&mut gpu, &mut mem, &map(23, 0), RESP_ERR_INVALID_PARAMETER);
}

#[test]
fn reset_reclaims_a_live_host_visible_mapping() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    negotiate_host_visible(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &blob(24, BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, 0, 4096, &[]),
        RESP_OK_NODATA,
    );
    assert_eq!(response_type(&gpu.execute_command(&mut mem, &map(24, 0))), RESP_OK_MAP_INFO);
    mem.write_bytes(VIRTIO_GPU_HOST_VISIBLE_BASE, &[7]).unwrap();
    gpu.write(&mut mem, 0x070, 0, 4);
    assert_eq!(mem.read(VIRTIO_GPU_HOST_VISIBLE_BASE, 1), Some(0));
}

fn negotiate_host_visible(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
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

fn blob(
    id: u32,
    memory: u32,
    flags: u32,
    blob_id: u64,
    size: u64,
    entries: &[(u64, u32)],
) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_CREATE_BLOB);
    for value in [id, memory, flags, entries.len() as u32] {
        push_u32(&mut request, value);
    }
    push_u64(&mut request, blob_id);
    push_u64(&mut request, size);
    for &(addr, len) in entries {
        push_u64(&mut request, addr);
        push_u32(&mut request, len);
        push_u32(&mut request, 0);
    }
    request
}

fn map(id: u32, offset: u64) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_MAP_BLOB);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    push_u64(&mut request, offset);
    request
}

fn unmap(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_UNMAP_BLOB);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn unref(id: u32) -> Vec<u8> {
    let mut request = header(CMD_RESOURCE_UNREF);
    push_u32(&mut request, id);
    push_u32(&mut request, 0);
    request
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, request: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, request)), expected);
}
