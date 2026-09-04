use super::super::super::DepthState;
use super::{geometry, texture_color, textured};
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;
use crate::devices::virtio_gpu::three_d::virgl::draw::TextureSnapshot;

pub(super) fn draw(
    resource: &mut GpuResource, rect: Rect, vertices: &[u8], texture: &TextureSnapshot,
    viewport: [f32; 6], scissor: Option<Rect>, state: DepthState, depth_values: &mut [f32],
) -> bool {
    if !texture_color::valid(vertices) || depth_values.len() * 4 != resource.pixels.len() { return false; }
    for triangle in vertices.chunks_exact(geometry::VERTICES * texture_color::STRIDE) {
        let (Some(points_3d), Some(colors), Some(uvs)) = (
            geometry::positions(triangle, texture_color::STRIDE), texture_color::colors(triangle), texture_color::uvs(triangle),
        ) else { return false; };
        let Some((points, bounds)) = geometry::setup(resource, rect, triangle, texture_color::STRIDE, viewport, scissor) else {
            return false;
        };
        for y in bounds.2..bounds.3 {
            for x in bounds.0..bounds.1 {
                let point = [x as f32 + 0.5, y as f32 + 0.5];
                if !geometry::contains(points, point[0], point[1]) { continue; }
                let weights = geometry::weights(points, point[0], point[1]);
                let value = weights.into_iter().zip(points_3d)
                    .map(|(weight, position)| weight * position[2]).sum::<f32>() * viewport[2] + viewport[5];
                let Some(index) = geometry::pixel(resource, rect, x, y).map(|index| index / 4) else { return false; };
                let Some(stored) = depth_values.get_mut(index) else { return false; };
                if !value.is_finite() || !(0.0..=1.0).contains(&value) || !stored.is_finite() { return false; }
                if state.compare.passes(value, *stored) {
                    if state.write { *stored = value; }
                    let Some(pixel) = resource.pixels.get_mut(index * 4..index * 4 + 4) else { return false; };
                    let sampled = textured::sample_one(texture, texture_color::interpolate(uvs, weights));
                    geometry::source_over(pixel, texture_color::multiply(sampled, texture_color::interpolate(colors, weights)));
                }
            }
        }
    }
    true
}
