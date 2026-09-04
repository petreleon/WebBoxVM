use super::VirglContext;

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct UniformBinding {
    pub resource: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) enum VertexConstants {
    InlineMatrix([u32; 16]),
    UniformMatrix(UniformBinding),
    UniformOffset(UniformBinding),
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
        self.vertex_constants = binding.map(VertexConstants::UniformOffset);
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_vertex_constants(
        &mut self,
        values: Option<[u32; 16]>,
    ) {
        self.vertex_constants = values.map(VertexConstants::InlineMatrix);
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_vertex_matrix_uniform(
        &mut self,
        binding: UniformBinding,
    ) {
        self.vertex_constants = Some(VertexConstants::UniformMatrix(binding));
    }

    pub(in crate::devices::virtio_gpu) fn remove_constant_resource(&mut self, resource: u32) {
        if matches!(
            self.fragment_constants,
            Some(FragmentConstants::Uniform(binding)) if binding.resource == resource
        ) {
            self.fragment_constants = None;
        }
        if matches!(self.vertex_constants,
            Some(VertexConstants::UniformMatrix(binding) | VertexConstants::UniformOffset(binding))
                if binding.resource == resource)
        {
            self.vertex_constants = None;
        }
    }
}
