use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) const VERTICES: usize = 3;

pub(super) fn positions(vertices: &[u8], stride: usize) -> Option<[[f32; 4]; VERTICES]> {
    if vertices.len() != VERTICES * stride || stride < 16 {
        return None;
    }
    let mut points = [[0.0; 4]; VERTICES];
    for (point, bytes) in points.iter_mut().zip(vertices.chunks_exact(stride)) {
        for (value, bytes) in point.iter_mut().zip(bytes[..16].chunks_exact(4)) {
            *value = f32::from_le_bytes(bytes.try_into().ok()?);
        }
    }
    Some(points)
}

pub(super) fn valid_positions(points: [[f32; 4]; VERTICES]) -> bool {
    points.iter().all(|point| {
        point.iter().all(|value| value.is_finite())
            && point[..3].iter().all(|value| (-1.0..=1.0).contains(value))
            && point[3] == 1.0
    }) && edge(
        [points[0][0], points[0][1]],
        [points[1][0], points[1][1]],
        [points[2][0], points[2][1]],
    )
    .abs()
        >= 0.001
}

pub(super) fn setup(
    resource: &GpuResource,
    rect: Rect,
    vertices: &[u8],
    stride: usize,
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> Option<([[f32; 2]; VERTICES], (u32, u32, u32, u32))> {
    let points = positions(vertices, stride)?;
    if !resource.is_texture_2d()
        || !rect.valid_within(resource.width, resource.height)
        || !valid_positions(points)
        || !valid_viewport(viewport, rect)
        || scissor.is_some_and(|scissor| !scissor.valid_within(rect.width, rect.height))
    {
        return None;
    }
    let points = points.map(|point| screen(point, viewport, rect.height));
    Some((points, bounds(points, rect, scissor)))
}

pub(super) fn contains(points: [[f32; 2]; VERTICES], x: f32, y: f32) -> bool {
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

pub(super) fn weights(points: [[f32; 2]; VERTICES], x: f32, y: f32) -> [f32; VERTICES] {
    let area = edge(points[0], points[1], points[2]);
    [
        edge(points[1], points[2], [x, y]) / area,
        edge(points[2], points[0], [x, y]) / area,
        edge(points[0], points[1], [x, y]) / area,
    ]
}

pub(super) fn pixel(resource: &GpuResource, rect: Rect, x: u32, y: u32) -> Option<usize> {
    usize::try_from((rect.y + y) * resource.width + rect.x + x)
        .ok()?
        .checked_mul(4)
}

pub(super) fn source_over(destination: &mut [u8], [red, green, blue, alpha]: [f32; 4]) {
    for (channel, source) in [blue, green, red].into_iter().enumerate() {
        destination[channel] =
            encode(source * alpha + f32::from(destination[channel]) / 255.0 * (1.0 - alpha));
    }
    destination[3] = encode(alpha + f32::from(destination[3]) / 255.0 * (1.0 - alpha));
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
    let axis = |index| points.map(|point| point[index]);
    let floor = |values: [f32; VERTICES], lower, upper| {
        values
            .into_iter()
            .fold(f32::INFINITY, |result, value| result.min(value))
            .floor()
            .clamp(lower, upper) as u32
    };
    let ceil = |values: [f32; VERTICES], lower, upper| {
        values
            .into_iter()
            .fold(f32::NEG_INFINITY, |result, value| result.max(value))
            .ceil()
            .clamp(lower, upper) as u32
    };
    (
        floor(axis(0), scissor.x as f32, max_x),
        ceil(axis(0), scissor.x as f32, max_x),
        floor(axis(1), scissor.y as f32, max_y),
        ceil(axis(1), scissor.y as f32, max_y),
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

fn edge(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> f32 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}

fn encode(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}
