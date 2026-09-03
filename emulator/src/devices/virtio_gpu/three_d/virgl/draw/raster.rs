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
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    let Some(points) = points(vertices) else {
        return false;
    };
    if !resource.is_texture_2d()
        || !rect.valid_within(resource.width, resource.height)
        || !valid(vertices)
        || !valid_viewport(viewport, rect)
        || scissor.is_some_and(|scissor| !scissor.valid_within(rect.width, rect.height))
    {
        return false;
    }
    let points = points.map(|point| screen(point, viewport, rect.height));
    let (min_x, max_x, min_y, max_y) = bounds(points, rect, scissor);
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
        && (-1.0..=1.0).contains(&point[2])
        && point[3] == 1.0
}

fn area(points: [[f32; 4]; VERTICES]) -> f32 {
    edge(
        [points[0][0], points[0][1]],
        [points[1][0], points[1][1]],
        [points[2][0], points[2][1]],
    )
}

fn screen(point: [f32; 4], viewport: [f32; 6], height: u32) -> [f32; 2] {
    [
        point[0] * viewport[0] + viewport[3],
        height as f32 - (point[1] * viewport[1] + viewport[4]),
    ]
}

fn bounds(points: [[f32; 2]; VERTICES], rect: Rect, scissor: Option<Rect>) -> (u32, u32, u32, u32) {
    let scissor = scissor.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: rect.width,
        height: rect.height,
    });
    let max_x = (scissor.x + scissor.width) as f32;
    let max_y = (scissor.y + scissor.height) as f32;
    let xs = points.map(|point| point[0]);
    let ys = points.map(|point| point[1]);
    (
        xs.iter()
            .fold(f32::INFINITY, |min, value| min.min(*value))
            .floor()
            .clamp(scissor.x as f32, max_x) as u32,
        xs.iter()
            .fold(f32::NEG_INFINITY, |max, value| max.max(*value))
            .ceil()
            .clamp(scissor.x as f32, max_x) as u32,
        ys.iter()
            .fold(f32::INFINITY, |min, value| min.min(*value))
            .floor()
            .clamp(scissor.y as f32, max_y) as u32,
        ys.iter()
            .fold(f32::NEG_INFINITY, |max, value| max.max(*value))
            .ceil()
            .clamp(scissor.y as f32, max_y) as u32,
    )
}

fn valid_viewport([sx, sy, sz, tx, ty, tz]: [f32; 6], rect: Rect) -> bool {
    [sx, sy, sz, tx, ty, tz].into_iter().all(f32::is_finite)
        && sx > 0.0
        && sy > 0.0
        && sz >= 0.0
        && tx - sx >= 0.0
        && tx + sx <= rect.width as f32
        && ty - sy >= 0.0
        && ty + sy <= rect.height as f32
        && tz - sz >= 0.0
        && tz + sz <= 1.0
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
