use super::VirglContext;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu) struct IndexBuffer {
    pub index_size: u32,
    pub offset: u32,
    pub resource: u32,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn set_index_buffer(
        &mut self,
        binding: Option<IndexBuffer>,
    ) {
        self.index_buffer = binding;
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn index_buffer(&self) -> Option<IndexBuffer> {
        self.index_buffer
    }

    pub(in crate::devices::virtio_gpu) fn remove_index_resource(&mut self, resource: u32) {
        if self
            .index_buffer
            .is_some_and(|binding| binding.resource == resource)
        {
            self.index_buffer = None;
        }
    }
}
