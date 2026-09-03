use super::VirglContext;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_sampler_view(
        &mut self,
        handle: u32,
        resource: u32,
    ) -> bool {
        handle != 0
            && self
                .pipeline
                .sampler_views
                .insert(handle, resource)
                .is_none()
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_sampler_view(
        &mut self,
        handle: u32,
    ) -> bool {
        if self.pipeline.sampler_views.remove(&handle).is_none() {
            return false;
        }
        if self.pipeline.bound_sampler_view == Some(handle) {
            self.pipeline.bound_sampler_view = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_sampler_view(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.pipeline.sampler_views.contains_key(&handle)) {
            return false;
        }
        self.pipeline.bound_sampler_view = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_sampler_state(
        &mut self,
        handle: u32,
    ) -> bool {
        handle != 0 && self.pipeline.sampler_states.insert(handle)
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_sampler_state(
        &mut self,
        handle: u32,
    ) -> bool {
        if !self.pipeline.sampler_states.remove(&handle) {
            return false;
        }
        if self.pipeline.bound_sampler_state == Some(handle) {
            self.pipeline.bound_sampler_state = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_sampler_state(
        &mut self,
        handle: Option<u32>,
    ) -> bool {
        if handle.is_some_and(|handle| !self.pipeline.sampler_states.contains(&handle)) {
            return false;
        }
        self.pipeline.bound_sampler_state = handle;
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn sampled_resource(&self) -> Option<u32> {
        self.pipeline.bound_sampler_state?;
        self.pipeline
            .bound_sampler_view
            .and_then(|handle| self.pipeline.sampler_views.get(&handle).copied())
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn bound_sampled_resource(&self) -> Option<u32> {
        self.sampled_resource()
    }

    pub(in crate::devices::virtio_gpu) fn remove_sampler_resource(&mut self, resource: u32) {
        self.pipeline
            .sampler_views
            .retain(|_, candidate| *candidate != resource);
        if self
            .pipeline
            .bound_sampler_view
            .is_some_and(|handle| !self.pipeline.sampler_views.contains_key(&handle))
        {
            self.pipeline.bound_sampler_view = None;
        }
    }
}
