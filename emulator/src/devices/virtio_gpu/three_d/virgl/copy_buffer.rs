use super::CopyRegion;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) fn valid(source: &GpuResource, destination: &GpuResource, copy: CopyRegion) -> bool {
    source.is_buffer()
        && destination.is_buffer()
        && source.format == destination.format
        && copy.dst_level == 0
        && copy.src_level == 0
        && copy.dst_y == 0
        && copy.dst_z == 0
        && copy.src_rect.y == 0
        && copy.src_z == 0
        && copy.src_rect.height == 1
        && copy.depth == 1
        && range(source, copy.src_rect.x, copy.src_rect.width).is_some()
        && range(destination, copy.dst_x, copy.src_rect.width).is_some()
}

pub(super) fn source_bytes(resource: &GpuResource, copy: CopyRegion) -> Option<Vec<u8>> {
    let (start, end) = range(resource, copy.src_rect.x, copy.src_rect.width)?;
    Some(resource.pixels.get(start..end)?.to_vec())
}

pub(super) fn write_bytes(
    resource: &mut GpuResource,
    copy: CopyRegion,
    bytes: &[u8],
) -> Option<()> {
    let (start, end) = range(resource, copy.dst_x, copy.src_rect.width)?;
    if bytes.len() != end.checked_sub(start)? {
        return None;
    }
    resource.pixels.get_mut(start..end)?.copy_from_slice(bytes);
    Some(())
}

fn range(resource: &GpuResource, x: u32, width: u32) -> Option<(usize, usize)> {
    if !resource.is_buffer() || width == 0 {
        return None;
    }
    let start = usize::try_from(x).ok()?;
    let len = usize::try_from(width).ok()?;
    start
        .checked_add(len)
        .filter(|end| *end <= resource.pixels.len())
        .map(|end| (start, end))
}
