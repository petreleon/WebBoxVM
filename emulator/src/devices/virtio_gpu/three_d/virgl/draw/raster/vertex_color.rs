use super::geometry;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

const STRIDE: usize = 32;

pub(super) fn valid(vertices: &[u8]) -> bool {
    geometry::valid_vertices(vertices, STRIDE)
        && vertices
            .chunks_exact(geometry::VERTICES * STRIDE)
            .all(|triangle| colors(triangle).is_some())
}

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    if !valid(vertices) {
        return false;
    }
    for triangle in vertices.chunks_exact(geometry::VERTICES * STRIDE) {
        let Some(colors) = colors(triangle) else { return false; };
        let Some((points, (min_x, max_x, min_y, max_y))) =
            geometry::setup(resource, rect, triangle, STRIDE, viewport, scissor)
        else { return false; };
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
                geometry::source_over(pixel, interpolate(colors, geometry::weights(points, point[0], point[1])));
            }
        }
    }
    true
}

fn colors(vertices: &[u8]) -> Option<[[f32; 4]; geometry::VERTICES]> {
    if vertices.len() != geometry::VERTICES * STRIDE {
        return None;
    }
    let mut values = [[0.0; 4]; geometry::VERTICES];
    for (color, vertex) in values.iter_mut().zip(vertices.chunks_exact(STRIDE)) {
        for (value, bytes) in color.iter_mut().zip(vertex[16..].chunks_exact(4)) {
            *value = f32::from_le_bytes(bytes.try_into().ok()?);
        }
    }
    values
        .into_iter()
        .flatten()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
        .then_some(values)
}

fn interpolate(
    colors: [[f32; 4]; geometry::VERTICES],
    weights: [f32; geometry::VERTICES],
) -> [f32; 4] {
    std::array::from_fn(|channel| {
        colors
            .iter()
            .zip(weights)
            .map(|(color, weight)| color[channel] * weight)
            .sum()
    })
}
