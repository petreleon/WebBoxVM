use super::{MAX_VIRGL_DRAW_INPUT_VERTICES, MAX_VIRGL_DRAW_VERTICES, TRIANGLE_VERTICES};

const PIPE_PRIM_TRIANGLES: u32 = 4;
const PIPE_PRIM_TRIANGLE_STRIP: u32 = 5;
const PIPE_PRIM_TRIANGLE_FAN: u32 = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu::three_d::virgl) enum Primitive {
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl Primitive {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn from_wire(value: u32) -> Option<Self> {
        match value {
            PIPE_PRIM_TRIANGLES => Some(Self::Triangles),
            PIPE_PRIM_TRIANGLE_STRIP => Some(Self::TriangleStrip),
            PIPE_PRIM_TRIANGLE_FAN => Some(Self::TriangleFan),
            _ => None,
        }
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn valid_count(self, count: u32) -> bool {
        (TRIANGLE_VERTICES..=MAX_VIRGL_DRAW_INPUT_VERTICES).contains(&count)
            && (self != Self::Triangles || count.is_multiple_of(TRIANGLE_VERTICES))
    }

    pub(super) fn output_count(self, count: u32) -> Option<u32> {
        if !self.valid_count(count) {
            return None;
        }
        let output = match self {
            Self::Triangles => Some(count),
            Self::TriangleStrip | Self::TriangleFan => count
                .checked_sub(TRIANGLE_VERTICES - 1)?
                .checked_mul(TRIANGLE_VERTICES),
        }?;
        (output <= MAX_VIRGL_DRAW_VERTICES).then_some(output)
    }

    pub(super) fn expand(self, indices: &[u32]) -> Option<Vec<u32>> {
        let count = u32::try_from(indices.len()).ok()?;
        let capacity = usize::try_from(self.output_count(count)?).ok()?;
        let mut output = Vec::with_capacity(capacity);
        match self {
            Self::Triangles => output.extend_from_slice(indices),
            Self::TriangleStrip => {
                for current in 2..indices.len() {
                    let [first, second] = [indices[current - 2], indices[current - 1]];
                    let triangle = if current.is_multiple_of(2) {
                        [first, second, indices[current]]
                    } else {
                        [second, first, indices[current]]
                    };
                    output.extend(triangle);
                }
            }
            Self::TriangleFan => {
                for current in 2..indices.len() {
                    output.extend([indices[0], indices[current - 1], indices[current]]);
                }
            }
        }
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_VIRGL_DRAW_VERTICES, Primitive};

    #[test]
    fn triangle_strip_expansion_preserves_alternating_winding() {
        assert_eq!(
            Primitive::TriangleStrip.expand(&[0, 1, 2, 3, 4]),
            Some(vec![0, 1, 2, 2, 1, 3, 2, 3, 4])
        );
    }

    #[test]
    fn triangle_strip_bounds_the_normalized_output() {
        assert_eq!(Primitive::TriangleStrip.output_count(3), Some(3));
        assert_eq!(Primitive::TriangleStrip.output_count(1023), Some(MAX_VIRGL_DRAW_VERTICES));
        assert_eq!(Primitive::TriangleStrip.output_count(1024), None);
    }

    #[test]
    fn triangle_fan_expansion_keeps_the_first_vertex_as_its_spoke() {
        assert_eq!(
            Primitive::TriangleFan.expand(&[0, 1, 2, 3, 4]),
            Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 4])
        );
    }
}
