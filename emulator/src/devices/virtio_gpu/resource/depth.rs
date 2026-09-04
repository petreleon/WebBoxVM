use super::{GpuResource, ResourceKind};

pub(in crate::devices::virtio_gpu) const FORMAT_Z32_FLOAT: u32 = 18;

impl GpuResource {
    pub(in crate::devices::virtio_gpu) fn new_depth_texture(width: u32, height: u32) -> Option<Self> {
        let len = Self::byte_len(width, height)?;
        Some(Self {
            format: FORMAT_Z32_FLOAT,
            width,
            height,
            pixels: vec![0; len],
            backing: Vec::new(),
            kind: ResourceKind::DepthTexture2d,
            sampleable: false,
            buffer_bind: None,
        })
    }
}
