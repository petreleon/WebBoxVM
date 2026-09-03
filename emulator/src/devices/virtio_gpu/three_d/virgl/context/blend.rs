use super::VirglContext;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_blend(
        &mut self,
        handle: u32,
    ) -> bool {
        handle != 0 && self.blend_states.insert(handle)
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_blend(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.blend_states.contains(&handle)) {
            return false;
        }
        self.bound_blend_state = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_blend(
        &mut self,
        handle: u32,
    ) -> bool {
        if !self.blend_states.remove(&handle) {
            return false;
        }
        if self.bound_blend_state == Some(handle) {
            self.bound_blend_state = None;
        }
        true
    }
}
