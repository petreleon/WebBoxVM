use super::*;

pub(super) fn read_image(image: &[u8], offset: u64, dst: &mut [u8]) -> u8 {
    if offset < image.len() as u64 {
        let available = ((image.len() as u64 - offset) as usize).min(dst.len());
        let start = offset as usize;
        dst[..available].copy_from_slice(&image[start..start + available]);
        dst[available..].fill(0);
    } else {
        dst.fill(0);
    }
    VIRTIO_BLK_S_OK
}

impl SparseDiskStorage {
    pub(super) fn write_in_range(&mut self, offset: u64, src: &[u8]) {
        let mut done = 0usize;
        while done < src.len() {
            let current = offset + done as u64;
            let chunk_index = current / SPARSE_DISK_CHUNK_SIZE as u64;
            let chunk_offset = (current % SPARSE_DISK_CHUNK_SIZE as u64) as usize;
            let count = (src.len() - done).min(SPARSE_DISK_CHUNK_SIZE - chunk_offset);
            let chunk = self
                .chunks
                .entry(chunk_index)
                .or_insert_with(|| Box::new([0; SPARSE_DISK_CHUNK_SIZE]));

            chunk[chunk_offset..chunk_offset + count].copy_from_slice(&src[done..done + count]);
            done += count;
        }
    }
}
