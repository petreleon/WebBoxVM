use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

const VERTEX_BYTES: usize = 16;
const VERTICES: usize = 3;

pub(super) fn valid(vertices: &[u8]) -> bool {
    let Some(points) = points(vertices) else {
        return false;
    };
    points.iter().all(valid_point) && area(points).abs() >= 0.001
}

pub(super) fn draw(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    color: [f32; 4],
) -> bool {
    let Some(points) = points(vertices) else {
        return false;
    };
    if !resource.is_texture_2d()
        || !rect.valid_within(resource.width, resource.height)
        || !valid(vertices)
    {
        return false;
    }
    let points = points.map(|point| screen(point, rect.width, rect.height));
    let (min_x, max_x, min_y, max_y) = bounds(points, rect);
    for y in min_y..max_y {
        for x in min_x..max_x {
            if contains(points, x as f32 + 0.5, y as f32 + 0.5) {
                let index = usize::try_from((rect.y + y) * resource.width + rect.x + x)
                    .ok()
                    .and_then(|pixel| pixel.checked_mul(4));
                let Some(index) = index else { return false };
                let Some(pixel) = resource.pixels.get_mut(index..index + 4) else {
                    return false;
                };
                source_over(pixel, color);
            }
        }
    }
    true
}

fn source_over(destination: &mut [u8], [red, green, blue, alpha]: [f32; 4]) {
    for (channel, source) in [blue, green, red].into_iter().enumerate() {
        let value = source * alpha + f32::from(destination[channel]) / 255.0 * (1.0 - alpha);
        destination[channel] = encode(value);
    }
    let value = alpha + f32::from(destination[3]) / 255.0 * (1.0 - alpha);
    destination[3] = encode(value);
}

fn encode(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn points(vertices: &[u8]) -> Option<[[f32; 4]; VERTICES]> {
    if vertices.len() != VERTICES * VERTEX_BYTES {
        return None;
    }
    let mut points = [[0.0; 4]; VERTICES];
    for (point, bytes) in points.iter_mut().zip(vertices.chunks_exact(VERTEX_BYTES)) {
        for (value, bytes) in point.iter_mut().zip(bytes.chunks_exact(4)) {
            *value = f32::from_le_bytes(bytes.try_into().ok()?);
        }
    }
    Some(points)
}

fn valid_point(point: &[f32; 4]) -> bool {
    point.iter().all(|value| value.is_finite())
        && (-1.0..=1.0).contains(&point[0])
        && (-1.0..=1.0).contains(&point[1])
        && (0.0..=1.0).contains(&point[2])
        && point[3] == 1.0
}

fn area(points: [[f32; 4]; VERTICES]) -> f32 {
    edge(
        [points[0][0], points[0][1]],
        [points[1][0], points[1][1]],
        [points[2][0], points[2][1]],
    )
}

fn screen(point: [f32; 4], width: u32, height: u32) -> [f32; 2] {
    [
        (point[0] * 0.5 + 0.5) * width as f32,
        (0.5 - point[1] * 0.5) * height as f32,
    ]
}

fn bounds(points: [[f32; 2]; VERTICES], rect: Rect) -> (u32, u32, u32, u32) {
    let xs = points.map(|point| point[0]);
    let ys = points.map(|point| point[1]);
    (
        xs.iter()
            .fold(f32::INFINITY, |min, value| min.min(*value))
            .floor()
            .clamp(0.0, rect.width as f32) as u32,
        xs.iter()
            .fold(f32::NEG_INFINITY, |max, value| max.max(*value))
            .ceil()
            .clamp(0.0, rect.width as f32) as u32,
        ys.iter()
            .fold(f32::INFINITY, |min, value| min.min(*value))
            .floor()
            .clamp(0.0, rect.height as f32) as u32,
        ys.iter()
            .fold(f32::NEG_INFINITY, |max, value| max.max(*value))
            .ceil()
            .clamp(0.0, rect.height as f32) as u32,
    )
}

fn contains(points: [[f32; 2]; VERTICES], x: f32, y: f32) -> bool {
    let edges = [
        edge(points[1], points[2], [x, y]),
        edge(points[2], points[0], [x, y]),
        edge(points[0], points[1], [x, y]),
    ];
    if edge(points[0], points[1], points[2]) > 0.0 {
        edges.into_iter().all(|value| value >= 0.0)
    } else {
        edges.into_iter().all(|value| value <= 0.0)
    }
}

fn edge(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> f32 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}
