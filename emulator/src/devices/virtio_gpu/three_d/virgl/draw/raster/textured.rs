use super::geometry;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;
use crate::devices::virtio_gpu::three_d::virgl::{
    SamplerAddressMode, SamplerFilter, draw::TextureSnapshot,
};

const STRIDE: usize = 24;

pub(super) fn valid(vertices: &[u8]) -> bool {
    geometry::valid_vertices(vertices, STRIDE)
        && vertices
            .chunks_exact(geometry::VERTICES * STRIDE)
            .all(|triangle| uvs(triangle).is_some())
}

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    textures: &[TextureSnapshot],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    if !(1..=2).contains(&textures.len()) || !valid(vertices) {
        return false;
    }
    for triangle in vertices.chunks_exact(geometry::VERTICES * STRIDE) {
        let Some(uvs) = uvs(triangle) else { return false; };
        let Some((points, (min_x, max_x, min_y, max_y))) =
            geometry::setup(resource, rect, triangle, STRIDE, viewport, scissor)
        else { return false; };
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
                        pixel, sample(textures, interpolate(uvs, geometry::weights(points, px, py))),
                    );
                }
            }
        }
    }
    true
}

pub(super) fn uvs(vertices: &[u8]) -> Option<[[f32; 2]; geometry::VERTICES]> {
    if vertices.len() != geometry::VERTICES * STRIDE {
        return None;
    }
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

pub(super) fn interpolate(
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

fn sample(textures: &[TextureSnapshot], uv: [f32; 2]) -> [f32; 4] {
    textures
        .iter()
        .map(|texture| sample_one(texture, uv))
        .fold([1.0; 4], |color, sample| {
            std::array::from_fn(|index| color[index] * sample[index])
        })
}

pub(super) fn sample_one(texture: &TextureSnapshot, [u, v]: [f32; 2]) -> [f32; 4] {
    let v = 1.0 - v;
    match texture.sampler.filter() {
        SamplerFilter::Nearest => color(
            texture,
            texel(u, texture.width, texture.sampler.address_mode()),
            texel(v, texture.height, texture.sampler.address_mode()),
        ),
        SamplerFilter::Linear => linear(texture, u, v),
    }
}

fn linear(texture: &TextureSnapshot, u: f32, v: f32) -> [f32; 4] {
    let (left, right, horizontal) = linear_texels(u, texture.width);
    let (top, bottom, vertical) = linear_texels(v, texture.height);
    mix(
        mix(
            color(texture, left, top),
            color(texture, right, top),
            horizontal,
        ),
        mix(
            color(texture, left, bottom),
            color(texture, right, bottom),
            horizontal,
        ),
        vertical,
    )
}

fn linear_texels(value: f32, size: u32) -> (usize, usize, f32) {
    let position = value * size as f32 - 0.5;
    let lower = position.floor();
    let last = size.saturating_sub(1) as f32;
    (
        lower.clamp(0.0, last) as usize,
        (lower + 1.0).clamp(0.0, last) as usize,
        position - lower,
    )
}

fn color(texture: &TextureSnapshot, x: usize, y: usize) -> [f32; 4] {
    let index = (y * texture.width as usize + x) * 4;
    let pixel = &texture.bgra[index..index + 4];
    [
        pixel[2] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[0] as f32 / 255.0,
        pixel[3] as f32 / 255.0,
    ]
}

fn mix(left: [f32; 4], right: [f32; 4], amount: f32) -> [f32; 4] {
    std::array::from_fn(|channel| left[channel] + (right[channel] - left[channel]) * amount)
}

fn texel(value: f32, size: u32, mode: SamplerAddressMode) -> usize {
    let value = match mode {
        SamplerAddressMode::ClampToEdge => value,
        SamplerAddressMode::Repeat => value - value.floor(),
    };
    (value * size as f32)
        .floor()
        .clamp(0.0, size.saturating_sub(1) as f32) as usize
}
