use super::VirglContext;
use crate::devices::virtio_gpu::resource::{FORMAT_R8_UNORM, FORMAT_R32G32_FLOAT, FORMAT_R32G32B32A32_FLOAT};
use std::collections::HashMap;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu::three_d::virgl) enum VertexLayout {
    Single(VertexElement),
    Textured,
    VertexColor,
}

impl VertexLayout {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn from_elements(elements: &[VertexElement]) -> Option<Self> {
        match elements {
            [element] => Some(Self::Single(*element)),
            [
                VertexElement {
                    offset: 0,
                    divisor: 0,
                    buffer_index: 0,
                    format: FORMAT_R32G32B32A32_FLOAT,
                },
                VertexElement {
                    offset: 16,
                    divisor: 0,
                    buffer_index: 0,
                    format: FORMAT_R32G32_FLOAT,
                },
            ] => Some(Self::Textured),
            [
                VertexElement { offset: 0, divisor: 0, buffer_index: 0, format: FORMAT_R32G32B32A32_FLOAT },
                VertexElement { offset: 16, divisor: 0, buffer_index: 0, format: FORMAT_R32G32B32A32_FLOAT },
            ] => Some(Self::VertexColor),
            _ => None,
        }
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn valid(self) -> bool {
        matches!(
            self,
            Self::Textured | Self::VertexColor
                | Self::Single(VertexElement {
                    offset: 0,
                    divisor: 0,
                    buffer_index: 0,
                    format: FORMAT_R8_UNORM | FORMAT_R32G32B32A32_FLOAT,
                })
        )
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn draw_stride(self) -> Option<u32> {
        match self {
            Self::Single(VertexElement {
                format: FORMAT_R32G32B32A32_FLOAT,
                ..
            }) => Some(16),
            Self::Textured => Some(24),
            Self::VertexColor => Some(32),
            Self::Single(_) => None,
        }
    }
    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn first(self) -> VertexElement {
        match self {
            Self::Single(element) => element,
            Self::Textured | Self::VertexColor => VertexElement {
                offset: 0,
                divisor: 0,
                buffer_index: 0,
                format: FORMAT_R32G32B32A32_FLOAT,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct VertexState {
    buffer: Option<VertexBuffer>,
    layouts: HashMap<u32, VertexLayout>,
    bound: Option<u32>,
}

impl VertexState {
    pub(super) fn new() -> Self {
        Self {
            buffer: None,
            layouts: HashMap::new(),
            bound: None,
        }
    }

    fn remove_resource(&mut self, resource: u32) {
        if self
            .buffer
            .is_some_and(|buffer| buffer.resource == resource)
        {
            self.buffer = None;
        }
    }
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn set_vertex_buffer(
        &mut self,
        binding: Option<VertexBuffer>,
    ) {
        self.vertex.buffer = binding;
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn vertex_buffer(&self) -> Option<VertexBuffer> {
        self.vertex.buffer
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_vertex_elements(&mut self, handle: u32, layout: VertexLayout) -> bool {
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
    ) -> Option<(VertexBuffer, VertexLayout)> {
        Some((
            self.vertex.buffer?,
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
