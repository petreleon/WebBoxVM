use super::VirglContext;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_depth_state(
        &mut self,
        handle: u32,
    ) -> bool {
        handle != 0 && self.pipeline.depth_states.insert(handle)
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_depth_state(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.pipeline.depth_states.contains(&handle)) {
            return false;
        }
        self.pipeline.bound_depth_state = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_depth_state(
        &mut self,
        handle: u32,
    ) -> bool {
        if !self.pipeline.depth_states.remove(&handle) {
            return false;
        }
        if self.pipeline.bound_depth_state == Some(handle) {
            self.pipeline.bound_depth_state = None;
        }
        true
    }
}
