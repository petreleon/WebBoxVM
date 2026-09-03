use super::{geometry, textured};
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;
use crate::devices::virtio_gpu::three_d::virgl::draw::TextureSnapshot;

const STRIDE: usize = 40;

pub(super) fn valid(vertices: &[u8]) -> bool {
    geometry::positions(vertices, STRIDE).is_some_and(geometry::valid_positions)
        && colors(vertices).is_some()
        && uvs(vertices).is_some()
}

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    texture: &TextureSnapshot,
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    let (Some(colors), Some(uvs)) = (colors(vertices), uvs(vertices)) else {
        return false;
    };
    let Some((points, (min_x, max_x, min_y, max_y))) =
        geometry::setup(resource, rect, vertices, STRIDE, viewport, scissor)
    else {
        return false;
    };
    for y in min_y..max_y {
        for x in min_x..max_x {
            let point = [x as f32 + 0.5, y as f32 + 0.5];
            if !geometry::contains(points, point[0], point[1]) {
                continue;
            }
            let Some(index) = geometry::pixel(resource, rect, x, y) else {
                return false;
            };
            let Some(pixel) = resource.pixels.get_mut(index..index + 4) else {
                return false;
            };
            let weights = geometry::weights(points, point[0], point[1]);
            let color = interpolate(colors, weights);
            let texture = textured::sample_one(texture, interpolate(uvs, weights));
            geometry::source_over(pixel, multiply(texture, color));
        }
    }
    true
}

fn colors(vertices: &[u8]) -> Option<[[f32; 4]; geometry::VERTICES]> {
    attributes(vertices, 16, 4, |value| (0.0..=1.0).contains(&value))
}

fn uvs(vertices: &[u8]) -> Option<[[f32; 2]; geometry::VERTICES]> {
    attributes(vertices, 32, 2, |value| (-8.0..=8.0).contains(&value))
}

fn attributes<const N: usize>(
    vertices: &[u8],
    offset: usize,
    count: usize,
    range: impl Fn(f32) -> bool,
) -> Option<[[f32; N]; geometry::VERTICES]> {
    (count == N && vertices.len() == geometry::VERTICES * STRIDE).then_some(())?;
    let mut values = [[0.0; N]; geometry::VERTICES];
    for (values, vertex) in values.iter_mut().zip(vertices.chunks_exact(STRIDE)) {
        for (value, bytes) in values.iter_mut().zip(vertex[offset..offset + N * 4].chunks_exact(4)) {
            *value = f32::from_le_bytes(bytes.try_into().ok()?);
        }
    }
    values.into_iter().flatten().all(|value| value.is_finite() && range(value)).then_some(values)
}

fn interpolate<const N: usize>(
    values: [[f32; N]; geometry::VERTICES],
    weights: [f32; geometry::VERTICES],
) -> [f32; N] {
    std::array::from_fn(|channel| values.iter().zip(weights)
        .map(|(value, weight)| value[channel] * weight).sum())
}

fn multiply(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    std::array::from_fn(|channel| left[channel] * right[channel])
}
