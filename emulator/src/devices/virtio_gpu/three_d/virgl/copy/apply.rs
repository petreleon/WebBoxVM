use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) fn source_pixels(resource: &GpuResource, rect: Rect) -> Option<Vec<u8>> {
    let row_len = usize::try_from(rect.width).ok()?.checked_mul(4)?;
    let rows = usize::try_from(rect.height).ok()?;
    let mut pixels = Vec::with_capacity(row_len.checked_mul(rows)?);
    for y in rect.y..rect.y.checked_add(rect.height)? {
        let start = pixel_offset(resource, rect.x, y)?;
        pixels.extend_from_slice(resource.pixels.get(start..start.checked_add(row_len)?)?);
    }
    Some(pixels)
}

pub(super) fn write_pixels(resource: &mut GpuResource, rect: Rect, pixels: &[u8]) -> Option<()> {
    let row_len = usize::try_from(rect.width).ok()?.checked_mul(4)?;
    if pixels.len() != row_len.checked_mul(usize::try_from(rect.height).ok()?)? { return None; }
    for (row, y) in (rect.y..rect.y.checked_add(rect.height)?).enumerate() {
        let start = pixel_offset(resource, rect.x, y)?;
        let source = row.checked_mul(row_len)?;
        let end = source.checked_add(row_len)?;
        resource.pixels.get_mut(start..start.checked_add(row_len)?)?.copy_from_slice(&pixels[source..end]);
    }
    Some(())
}

fn pixel_offset(resource: &GpuResource, x: u32, y: u32) -> Option<usize> {
    usize::try_from(y).ok()?.checked_mul(usize::try_from(resource.width).ok()?)?
        .checked_add(usize::try_from(x).ok()?)?.checked_mul(4)
}
