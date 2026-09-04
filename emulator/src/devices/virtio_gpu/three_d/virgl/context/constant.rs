use super::VirglContext;

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) enum FragmentConstants {
    Inline([u32; 4]),
    Uniform { resource: u32, offset: u32 },
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_fragment_constants(
        &mut self,
        values: Option<[u32; 4]>,
    ) {
        self.fragment_constants = values.map(FragmentConstants::Inline);
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_fragment_uniform(
        &mut self,
        resource: u32,
        offset: u32,
    ) {
        self.fragment_constants = Some(FragmentConstants::Uniform { resource, offset });
    }

    pub(in crate::devices::virtio_gpu) fn remove_constant_resource(&mut self, resource: u32) {
        if matches!(
            self.fragment_constants,
            Some(FragmentConstants::Uniform { resource: bound, .. }) if bound == resource
        ) {
            self.fragment_constants = None;
        }
    }
}
