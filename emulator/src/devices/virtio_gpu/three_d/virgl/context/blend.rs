use super::{BlendMode, VirglContext};

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_blend(
        &mut self,
        handle: u32,
        blend: BlendMode,
    ) -> bool {
        if handle == 0 || self.pipeline.blend_states.contains_key(&handle) { return false; }
        self.pipeline.blend_states.insert(handle, blend);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_blend(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.pipeline.blend_states.contains_key(&handle)) {
            return false;
        }
        self.pipeline.bound_blend_state = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_blend(
        &mut self,
        handle: u32,
    ) -> bool {
        if self.pipeline.blend_states.remove(&handle).is_none() {
            return false;
        }
        if self.pipeline.bound_blend_state == Some(handle) {
            self.pipeline.bound_blend_state = None;
        }
        true
    }
}
