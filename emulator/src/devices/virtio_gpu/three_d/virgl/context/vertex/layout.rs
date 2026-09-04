use crate::devices::virtio_gpu::resource::{
    FORMAT_R8_UNORM, FORMAT_R32G32_FLOAT, FORMAT_R32G32B32A32_FLOAT,
};

pub(in crate::devices::virtio_gpu::three_d::virgl) const MAX_VIRGL_VERTEX_BUFFERS: usize = 3;

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
    Textured([VertexElement; 2]),
    VertexColor([VertexElement; 2]),
    TextureColor([VertexElement; 3]),
}

impl VertexLayout {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn from_elements(
        elements: &[VertexElement],
    ) -> Option<Self> {
        let layout = match elements {
            [position]
                if matches!(
                    position.format,
                    FORMAT_R8_UNORM | FORMAT_R32G32B32A32_FLOAT
                ) =>
            {
                Self::Single(*position)
            }
            [position, generic]
                if position.format == FORMAT_R32G32B32A32_FLOAT
                    && generic.format == FORMAT_R32G32_FLOAT =>
            {
                Self::Textured([*position, *generic])
            }
            [position, color]
                if position.format == FORMAT_R32G32B32A32_FLOAT
                    && color.format == FORMAT_R32G32B32A32_FLOAT =>
            {
                Self::VertexColor([*position, *color])
            }
            [position, color, uv]
                if position.format == FORMAT_R32G32B32A32_FLOAT
                    && color.format == FORMAT_R32G32B32A32_FLOAT
                    && uv.format == FORMAT_R32G32_FLOAT =>
            {
                Self::TextureColor([*position, *color, *uv])
            }
            _ => return None,
        };
        layout.valid().then_some(layout)
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn valid(self) -> bool {
        self.elements().iter().all(|element| {
            element.divisor == 0
                && usize::try_from(element.buffer_index)
                    .ok()
                    .is_some_and(|slot| slot < MAX_VIRGL_VERTEX_BUFFERS)
                && element.offset.is_multiple_of(format_bytes(element.format).unwrap_or(1))
        })
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn elements(&self) -> &[VertexElement] {
        match self {
            Self::Single(element) => std::slice::from_ref(element),
            Self::Textured(elements) | Self::VertexColor(elements) => elements,
            Self::TextureColor(elements) => elements,
        }
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn normalized_stride(self) -> Option<usize> {
        self.elements().iter().try_fold(0usize, |size, element| {
            size.checked_add(usize::try_from(format_bytes(element.format)?).ok()?)
        })
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn slot_stride(self, slot: usize) -> Option<u32> {
        let mut stride = None;
        for element in self.elements() {
            if usize::try_from(element.buffer_index).ok()? == slot {
                let end = element.offset.checked_add(format_bytes(element.format)?)?;
                stride = Some(stride.unwrap_or(0).max(end));
            }
        }
        stride
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn slot_format(self, slot: usize) -> Option<u32> {
        let mut count = 0;
        let mut format = None;
        for element in self.elements() {
            if usize::try_from(element.buffer_index).ok()? == slot {
                count += 1;
                format = Some(element.format);
            }
        }
        match count {
            0 => None,
            1 => format,
            _ => Some(FORMAT_R32G32B32A32_FLOAT),
        }
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn first(self) -> VertexElement {
        self.elements()[0]
    }
}

fn format_bytes(format: u32) -> Option<u32> {
    match format {
        FORMAT_R8_UNORM => Some(1),
        FORMAT_R32G32_FLOAT => Some(8),
        FORMAT_R32G32B32A32_FLOAT => Some(16),
        _ => None,
    }
}
