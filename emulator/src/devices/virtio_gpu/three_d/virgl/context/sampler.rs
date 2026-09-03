use super::super::MAX_VIRGL_FRAGMENT_SAMPLERS;
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
        self.pipeline
            .bound_sampler_views
            .iter_mut()
            .filter(|bound| **bound == Some(handle))
            .for_each(|bound| *bound = None);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_sampler_views(
        &mut self,
        start: usize,
        handles: &[Option<u32>],
    ) -> bool {
        let Some(end) = start.checked_add(handles.len()) else {
            return false;
        };
        if handles.is_empty()
            || end > MAX_VIRGL_FRAGMENT_SAMPLERS
            || handles
                .iter()
                .flatten()
                .any(|handle| !self.pipeline.sampler_views.contains_key(handle))
        {
            return false;
        }
        self.pipeline.bound_sampler_views[start..end].copy_from_slice(handles);
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
        self.pipeline
            .bound_sampler_states
            .iter_mut()
            .filter(|bound| **bound == Some(handle))
            .for_each(|bound| *bound = None);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_sampler_states(
        &mut self,
        start: usize,
        handles: &[Option<u32>],
    ) -> bool {
        let Some(end) = start.checked_add(handles.len()) else {
            return false;
        };
        if handles.is_empty()
            || end > MAX_VIRGL_FRAGMENT_SAMPLERS
            || handles
                .iter()
                .flatten()
                .any(|handle| !self.pipeline.sampler_states.contains(handle))
        {
            return false;
        }
        self.pipeline.bound_sampler_states[start..end].copy_from_slice(handles);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn sampled_resources(
        &self,
    ) -> [Option<u32>; MAX_VIRGL_FRAGMENT_SAMPLERS] {
        std::array::from_fn(|slot| {
            self.pipeline.bound_sampler_states[slot]?;
            self.pipeline.bound_sampler_views[slot]
                .and_then(|handle| self.pipeline.sampler_views.get(&handle).copied())
        })
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn bound_sampled_resource(&self) -> Option<u32> {
        self.sampled_resources()[0]
    }

    pub(in crate::devices::virtio_gpu) fn remove_sampler_resource(&mut self, resource: u32) {
        self.pipeline
            .sampler_views
            .retain(|_, candidate| *candidate != resource);
        self.pipeline
            .bound_sampler_views
            .iter_mut()
            .filter(|bound| {
                bound.is_some_and(|handle| !self.pipeline.sampler_views.contains_key(&handle))
            })
            .for_each(|bound| *bound = None);
    }
}
