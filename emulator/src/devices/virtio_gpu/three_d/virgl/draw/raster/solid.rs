use super::geometry;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

const STRIDE: usize = 16;

pub(super) fn valid(vertices: &[u8]) -> bool {
    geometry::positions(vertices, STRIDE).is_some_and(geometry::valid_positions)
}

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    color: [f32; 4],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    let Some((points, (min_x, max_x, min_y, max_y))) =
        geometry::setup(resource, rect, vertices, STRIDE, viewport, scissor)
    else {
        return false;
    };
    for y in min_y..max_y {
        for x in min_x..max_x {
            if geometry::contains(points, x as f32 + 0.5, y as f32 + 0.5) {
                let Some(index) = geometry::pixel(resource, rect, x, y) else {
                    return false;
                };
                let Some(pixel) = resource.pixels.get_mut(index..index + 4) else {
                    return false;
                };
                geometry::source_over(pixel, color);
            }
        }
    }
    true
}
