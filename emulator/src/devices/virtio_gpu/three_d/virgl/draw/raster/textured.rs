use super::geometry;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;
use crate::devices::virtio_gpu::three_d::virgl::draw::TextureSnapshot;

const STRIDE: usize = 24;

pub(super) fn valid(vertices: &[u8]) -> bool {
    geometry::positions(vertices, STRIDE).is_some_and(geometry::valid_positions)
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
    let Some(uvs) = uvs(vertices) else {
        return false;
    };
    let Some((points, (min_x, max_x, min_y, max_y))) =
        geometry::setup(resource, rect, vertices, STRIDE, viewport, scissor)
    else {
        return false;
    };
    for y in min_y..max_y {
        for x in min_x..max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            if geometry::contains(points, px, py) {
                let Some(index) = geometry::pixel(resource, rect, x, y) else {
                    return false;
                };
                let Some(pixel) = resource.pixels.get_mut(index..index + 4) else {
                    return false;
                };
                geometry::source_over(
                    pixel,
                    sample(texture, interpolate(uvs, geometry::weights(points, px, py))),
                );
            }
        }
    }
    true
}

fn uvs(vertices: &[u8]) -> Option<[[f32; 2]; geometry::VERTICES]> {
    let mut values = [[0.0; 2]; geometry::VERTICES];
    for (uv, vertex) in values.iter_mut().zip(vertices.chunks_exact(STRIDE)) {
        for (value, bytes) in uv.iter_mut().zip(vertex[16..24].chunks_exact(4)) {
            *value = f32::from_le_bytes(bytes.try_into().ok()?);
        }
    }
    values
        .into_iter()
        .flatten()
        .all(|value| value.is_finite() && (-8.0..=8.0).contains(&value))
        .then_some(values)
}

fn interpolate(
    uvs: [[f32; 2]; geometry::VERTICES],
    weights: [f32; geometry::VERTICES],
) -> [f32; 2] {
    [
        uvs.iter()
            .zip(weights)
            .map(|(uv, weight)| uv[0] * weight)
            .sum(),
        uvs.iter()
            .zip(weights)
            .map(|(uv, weight)| uv[1] * weight)
            .sum(),
    ]
}

fn sample(texture: &TextureSnapshot, [u, v]: [f32; 2]) -> [f32; 4] {
    let x = (u * texture.width as f32)
        .floor()
        .clamp(0.0, texture.width.saturating_sub(1) as f32) as usize;
    let y = ((1.0 - v) * texture.height as f32)
        .floor()
        .clamp(0.0, texture.height.saturating_sub(1) as f32) as usize;
    let index = (y * texture.width as usize + x) * 4;
    let pixel = &texture.bgra[index..index + 4];
    [
        pixel[2] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[0] as f32 / 255.0,
        pixel[3] as f32 / 255.0,
    ]
}
