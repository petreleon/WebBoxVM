use super::geometry;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

const STRIDE: usize = 16;

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    color: [f32; 4],
    viewport: [f32; 6],
    scissor: Option<Rect>,
    depth_values: &mut [f32],
) -> bool {
    if !geometry::valid_vertices(vertices, STRIDE) || depth_values.len() * 4 != resource.pixels.len() {
        return false;
    }
    for triangle in vertices.chunks_exact(geometry::VERTICES * STRIDE) {
        let Some(points_3d) = geometry::positions(triangle, STRIDE) else { return false; };
        let Some((points, bounds)) = geometry::setup(resource, rect, triangle, STRIDE, viewport, scissor) else {
            return false;
        };
        for y in bounds.2..bounds.3 {
            for x in bounds.0..bounds.1 {
                if !geometry::contains(points, x as f32 + 0.5, y as f32 + 0.5) { continue; }
                let weights = geometry::weights(points, x as f32 + 0.5, y as f32 + 0.5);
                let value = weights.into_iter().zip(points_3d).map(|(weight, point)| weight * point[2]).sum::<f32>() * viewport[2] + viewport[5];
                let Some(index) = geometry::pixel(resource, rect, x, y).map(|index| index / 4) else { return false; };
                let Some(stored) = depth_values.get_mut(index) else { return false; };
                if !value.is_finite() || !(0.0..=1.0).contains(&value) || !stored.is_finite() { return false; }
                if value < *stored {
                    let Some(pixel) = resource.pixels.get_mut(index * 4..index * 4 + 4) else { return false; };
                    *stored = value;
                    geometry::source_over(pixel, color);
                }
            }
        }
    }
    true
}
