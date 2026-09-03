mod blend;
mod draw;
mod shader;

use super::shader::Shader;
use std::collections::{HashMap, HashSet};

pub(in crate::devices::virtio_gpu) use draw::DrawState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu) struct VertexBuffer {
    pub stride: u32,
    pub offset: u32,
    pub resource: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu) struct VertexElement {
    pub offset: u32,
    pub divisor: u32,
    pub buffer_index: u32,
    pub format: u32,
}

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) struct VirglContext {
    pub(in crate::devices::virtio_gpu) generation: u32,
    attached: HashSet<u32>,
    framebuffer: Option<u32>,
    surfaces: HashMap<u32, u32>,
    blend_states: HashSet<u32>,
    bound_blend_state: Option<u32>,
    vertex_buffer: Option<VertexBuffer>,
    vertex_elements: HashMap<u32, VertexElement>,
    bound_vertex_elements: Option<u32>,
    shaders: HashMap<u32, Shader>,
    bound_vertex_shader: Option<u32>,
    bound_fragment_shader: Option<u32>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn new(generation: u32) -> Self {
        Self {
            generation,
            attached: HashSet::new(),
            framebuffer: None,
            surfaces: HashMap::new(),
            blend_states: HashSet::new(),
            bound_blend_state: None,
            vertex_buffer: None,
            vertex_elements: HashMap::new(),
            bound_vertex_elements: None,
            shaders: HashMap::new(),
            bound_vertex_shader: None,
            bound_fragment_shader: None,
        }
    }

    pub(in crate::devices::virtio_gpu) fn attach(&mut self, resource_id: u32) {
        self.attached.insert(resource_id);
    }

    pub(in crate::devices::virtio_gpu) fn detach(&mut self, resource_id: u32) -> bool {
        if !self.attached.remove(&resource_id) {
            return false;
        }
        if self
            .framebuffer
            .is_some_and(|handle| self.surface_resource(handle) == Some(resource_id))
        {
            self.framebuffer = None;
        }
        self.surfaces.retain(|_, resource| *resource != resource_id);
        if self
            .vertex_buffer
            .is_some_and(|binding| binding.resource == resource_id)
        {
            self.vertex_buffer = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu) fn has_surface(&self, handle: u32) -> bool {
        self.surfaces.contains_key(&handle)
    }

    pub(in crate::devices::virtio_gpu) fn is_attached(&self, resource_id: u32) -> bool {
        self.attached.contains(&resource_id)
    }

    pub(in crate::devices::virtio_gpu) fn add_surface(&mut self, handle: u32, resource: u32) {
        self.surfaces.insert(handle, resource);
    }

    pub(in crate::devices::virtio_gpu) fn destroy_surface(&mut self, handle: u32) -> bool {
        if self.surfaces.remove(&handle).is_none() {
            return false;
        }
        if self.framebuffer == Some(handle) {
            self.framebuffer = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu) fn bind_framebuffer(
        &mut self,
        surface: Option<u32>,
    ) -> bool {
        if surface.is_some_and(|handle| !self.has_surface(handle)) {
            return false;
        }
        self.framebuffer = surface;
        true
    }

    pub(in crate::devices::virtio_gpu) fn surface_resource(&self, handle: u32) -> Option<u32> {
        self.surfaces.get(&handle).copied()
    }

    pub(in crate::devices::virtio_gpu) fn framebuffer_resource(&self) -> Option<u32> {
        self.framebuffer
            .and_then(|handle| self.surface_resource(handle))
    }

    pub(in crate::devices::virtio_gpu) fn set_vertex_buffer(
        &mut self,
        binding: Option<VertexBuffer>,
    ) {
        self.vertex_buffer = binding;
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn vertex_buffer(&self) -> Option<VertexBuffer> {
        self.vertex_buffer
    }

    pub(in crate::devices::virtio_gpu) fn create_vertex_elements(
        &mut self,
        handle: u32,
        element: VertexElement,
    ) -> bool {
        if handle == 0 || self.vertex_elements.contains_key(&handle) {
            return false;
        }
        self.vertex_elements.insert(handle, element);
        true
    }

    pub(in crate::devices::virtio_gpu) fn bind_vertex_elements(&mut self, handle: u32) -> bool {
        if !self.vertex_elements.contains_key(&handle) {
            return false;
        }
        self.bound_vertex_elements = Some(handle);
        true
    }

    pub(in crate::devices::virtio_gpu) fn destroy_vertex_elements(&mut self, handle: u32) -> bool {
        if self.vertex_elements.remove(&handle).is_none() {
            return false;
        }
        if self.bound_vertex_elements == Some(handle) {
            self.bound_vertex_elements = None;
        }
        true
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn bound_vertex_element(&self) -> Option<VertexElement> {
        self.bound_vertex_elements
            .and_then(|handle| self.vertex_elements.get(&handle).copied())
    }

    pub(in crate::devices::virtio_gpu) fn remove_resource(&mut self, resource_id: u32) {
        let _ = self.detach(resource_id);
    }
}
