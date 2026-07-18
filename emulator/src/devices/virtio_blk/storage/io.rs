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
            let src_range = &src[done..done + count];
            let base_chunk = self
                .base
                .as_ref()
                .and_then(|base| base.chunk_data(chunk_index));
            if !self.overlay.contains_key(&chunk_index) {
                let base_range_has_data = base_chunk.is_some_and(|chunk| {
                    bytes_have_data(&chunk[chunk_offset..chunk_offset + count])
                });
                if !bytes_have_data(src_range) && !base_range_has_data {
                    done += count;
                    continue;
                }

                let mut initial = Box::new([0; SPARSE_DISK_CHUNK_SIZE]);
                if let Some(chunk) = base_chunk {
                    initial.copy_from_slice(chunk);
                }
                self.overlay.insert(chunk_index, initial);
            }

            let chunk = self
                .overlay
                .get_mut(&chunk_index)
                .expect("copy-on-write chunk was inserted");
            chunk[chunk_offset..chunk_offset + count].copy_from_slice(src_range);
            let matches_base = base_chunk.is_some_and(|base| chunk.as_ref() == base);
            if matches_base || (base_chunk.is_none() && !chunk_has_data(chunk)) {
                self.overlay.remove(&chunk_index);
            }
            done += count;
        }
    }
}
