use super::super::feature::STATUS_FEATURES_OK;
use super::super::{VIRTIO_F_VERSION_1, VIRTIO_GPU_F_CONTEXT_INIT, VIRTIO_GPU_F_VIRGL, VirtioGpu};
use crate::memory::PhysicalMemory;

#[test]
fn features_ok_requires_an_exact_two_page_feature_contract() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    write_features(
        &mut gpu,
        &mut mem,
        VIRTIO_GPU_F_VIRGL | VIRTIO_GPU_F_CONTEXT_INIT,
        VIRTIO_F_VERSION_1 >> 32,
    );
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(STATUS_FEATURES_OK.into()));
}

#[test]
fn unsupported_or_incomplete_features_clear_features_ok() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    write_features(
        &mut gpu,
        &mut mem,
        VIRTIO_GPU_F_VIRGL | (1 << 19),
        VIRTIO_F_VERSION_1 >> 32,
    );
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(0));

    write_features(&mut gpu, &mut mem, VIRTIO_GPU_F_VIRGL, 0);
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(0));
}

#[test]
fn changing_an_accepted_feature_page_revalidates_the_contract() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    write_features(
        &mut gpu,
        &mut mem,
        VIRTIO_GPU_F_CONTEXT_INIT,
        VIRTIO_F_VERSION_1 >> 32,
    );
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(STATUS_FEATURES_OK.into()));

    gpu.write(&mut mem, 0x024, 0, 4);
    gpu.write(&mut mem, 0x020, VIRTIO_GPU_F_CONTEXT_INIT | (1 << 31), 4);
    assert_eq!(gpu.read(0x070, 4), Some(0));
}

#[test]
fn reset_discards_feature_pages_before_another_negotiation() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    write_features(
        &mut gpu,
        &mut mem,
        VIRTIO_GPU_F_VIRGL,
        VIRTIO_F_VERSION_1 >> 32,
    );
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(STATUS_FEATURES_OK.into()));

    gpu.write(&mut mem, 0x070, 0, 4);
    gpu.write(&mut mem, 0x024, 0, 4);
    gpu.write(&mut mem, 0x020, VIRTIO_GPU_F_VIRGL, 4);
    gpu.write(&mut mem, 0x070, STATUS_FEATURES_OK.into(), 4);
    assert_eq!(gpu.read(0x070, 4), Some(0));
}

fn write_features(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, lower: u64, upper: u64) {
    gpu.write(&mut *mem, 0x024, 0, 4);
    gpu.write(&mut *mem, 0x020, lower, 4);
    gpu.write(&mut *mem, 0x024, 1, 4);
    gpu.write(&mut *mem, 0x020, upper, 4);
}
