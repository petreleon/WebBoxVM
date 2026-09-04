mod layout;

use super::VirglContext;
use std::collections::HashMap;

pub(in crate::devices::virtio_gpu::three_d::virgl) use layout::{
    MAX_VIRGL_VERTEX_BUFFERS, VertexLayout,
};
pub(in crate::devices::virtio_gpu) use layout::{VertexBuffer, VertexElement};

#[derive(Clone, Debug)]
pub(super) struct VertexState {
    buffers: [Option<VertexBuffer>; MAX_VIRGL_VERTEX_BUFFERS],
    layouts: HashMap<u32, VertexLayout>,
    bound: Option<u32>,
}

impl VertexState {
    pub(super) fn new() -> Self {
        Self {
            buffers: [None; MAX_VIRGL_VERTEX_BUFFERS],
            layouts: HashMap::new(),
            bound: None,
        }
    }

    fn remove_resource(&mut self, resource: u32) {
        for binding in &mut self.buffers {
            if binding.is_some_and(|buffer| buffer.resource == resource) {
                *binding = None;
            }
        }
    }
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn set_vertex_buffers(
        &mut self,
        bindings: [Option<VertexBuffer>; MAX_VIRGL_VERTEX_BUFFERS],
    ) {
        self.vertex.buffers = bindings;
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn vertex_buffer(&self) -> Option<VertexBuffer> {
        self.vertex.buffers[0]
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_vertex_elements(
        &mut self,
        handle: u32,
        layout: VertexLayout,
    ) -> bool {
        handle != 0 && self.vertex.layouts.insert(handle, layout).is_none()
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_vertex_elements(
        &mut self,
        handle: u32,
    ) -> bool {
        if !self.vertex.layouts.contains_key(&handle) {
            return false;
        }
        self.vertex.bound = Some(handle);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_vertex_elements(
        &mut self,
        handle: u32,
    ) -> bool {
        if self.vertex.layouts.remove(&handle).is_none() {
            return false;
        }
        if self.vertex.bound == Some(handle) {
            self.vertex.bound = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn draw_vertex_state(
        &self,
    ) -> Option<([Option<VertexBuffer>; MAX_VIRGL_VERTEX_BUFFERS], VertexLayout)> {
        Some((
            self.vertex.buffers,
            *self.vertex.layouts.get(&self.vertex.bound?)?,
        ))
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn bound_vertex_element(&self) -> Option<VertexElement> {
        self.vertex
            .bound
            .and_then(|handle| self.vertex.layouts.get(&handle).copied())
            .map(VertexLayout::first)
    }

    pub(in crate::devices::virtio_gpu) fn remove_vertex_resource(&mut self, resource: u32) {
        self.vertex.remove_resource(resource);
    }
}
