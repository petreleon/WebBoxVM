use super::VirglContext;
use super::pipeline::{Rasterizer, Viewport};
use crate::devices::virtio_gpu::protocol::Rect;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_rasterizer(
        &mut self,
        handle: u32,
        scissor: bool,
    ) -> bool {
        handle != 0
            && self
                .pipeline
                .rasterizers
                .insert(handle, Rasterizer { scissor })
                .is_none()
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_rasterizer(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.pipeline.rasterizers.contains_key(&handle)) {
            return false;
        }
        self.pipeline.bound_rasterizer = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_rasterizer(
        &mut self,
        handle: u32,
    ) -> bool {
        if self.pipeline.rasterizers.remove(&handle).is_none() {
            return false;
        }
        if self.pipeline.bound_rasterizer == Some(handle) {
            self.pipeline.bound_rasterizer = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_viewport(
        &mut self,
        viewport: Viewport,
    ) {
        self.pipeline.viewport = Some(viewport);
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_scissor(&mut self, scissor: Rect) {
        self.pipeline.scissor = Some(scissor);
    }
}
