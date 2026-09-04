use super::VirglContext;

pub(in crate::devices::virtio_gpu::three_d::virgl) type VertexConstants = [u32; 16];

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct UniformBinding {
    pub resource: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) enum FragmentConstants {
    Inline([u32; 4]),
    Uniform(UniformBinding),
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
        self.fragment_constants = Some(FragmentConstants::Uniform(UniformBinding {
            resource,
            offset,
        }));
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_vertex_uniform(
        &mut self,
        binding: Option<UniformBinding>,
    ) {
        self.vertex_uniform = binding;
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_vertex_constants(
        &mut self,
        values: Option<VertexConstants>,
    ) {
        self.vertex_constants = values;
    }

    pub(in crate::devices::virtio_gpu) fn remove_constant_resource(&mut self, resource: u32) {
        if matches!(
            self.fragment_constants,
            Some(FragmentConstants::Uniform(binding)) if binding.resource == resource
        ) {
            self.fragment_constants = None;
        }
        if self
            .vertex_uniform
            .is_some_and(|binding| binding.resource == resource)
        {
            self.vertex_uniform = None;
        }
    }
}
