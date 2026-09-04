use super::VirglContext;

#[derive(Clone, Copy, Debug)]
pub(super) struct Surface {
    pub(super) resource: u32,
    pub(super) depth: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Framebuffer {
    pub(super) color: u32,
    pub(super) depth: Option<u32>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn has_surface(&self, handle: u32) -> bool {
        self.surfaces.contains_key(&handle)
    }

    pub(in crate::devices::virtio_gpu) fn add_surface(
        &mut self,
        handle: u32,
        resource: u32,
        depth: bool,
    ) {
        self.surfaces.insert(handle, Surface { resource, depth });
    }

    pub(in crate::devices::virtio_gpu) fn destroy_surface(&mut self, handle: u32) -> bool {
        if self.surfaces.remove(&handle).is_none() {
            return false;
        }
        if self.framebuffer.is_some_and(|frame| frame.color == handle || frame.depth == Some(handle)) {
            self.framebuffer = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu) fn bind_framebuffer(
        &mut self,
        color: Option<u32>,
        depth: Option<u32>,
    ) -> bool {
        let Some(color) = color else {
            if depth.is_none() { self.framebuffer = None; return true; }
            return false;
        };
        if !self.surfaces.get(&color).is_some_and(|surface| !surface.depth)
            || !depth.is_none_or(|handle| self.surfaces.get(&handle).is_some_and(|surface| surface.depth))
        {
            return false;
        }
        self.framebuffer = Some(Framebuffer { color, depth });
        true
    }

    pub(in crate::devices::virtio_gpu) fn surface_resource(&self, handle: u32) -> Option<u32> {
        self.surfaces.get(&handle).map(|surface| surface.resource)
    }

    pub(in crate::devices::virtio_gpu) fn framebuffer_resource(&self) -> Option<u32> {
        self.framebuffer.and_then(|frame| self.surface_resource(frame.color))
    }

    pub(in crate::devices::virtio_gpu) fn framebuffer_depth_resource(&self) -> Option<u32> {
        self.framebuffer
            .and_then(|frame| frame.depth.and_then(|handle| self.surface_resource(handle)))
    }

    pub(super) fn remove_framebuffer_resource(&mut self, resource: u32) {
        if self.framebuffer.is_some_and(|frame| {
            self.surface_resource(frame.color) == Some(resource)
                || frame.depth.is_some_and(|handle| self.surface_resource(handle) == Some(resource))
        }) {
            self.framebuffer = None;
        }
    }
}
