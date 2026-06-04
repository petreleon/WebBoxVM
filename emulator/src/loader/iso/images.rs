use crate::constants::ARM64_KERNEL_MAGIC;
use flate2::read::GzDecoder;
use std::io::Read;

pub(super) fn prepare_kernel_image(data: &[u8]) -> Result<Vec<u8>, String> {
    let kernel = decompress_if_gzip(data, "kernel")?;
    if !looks_like_arm64_image(&kernel) {
        return Err(
            "ISO kernel is not an uncompressed ARM64 Linux Image; x86 ISOs are not supported"
                .to_string(),
        );
    }
    Ok(kernel)
}

pub(super) fn prepare_initrd_image(data: &[u8]) -> Result<Vec<u8>, String> {
    decompress_if_gzip(data, "initrd")
}

fn decompress_if_gzip(data: &[u8], label: &str) -> Result<Vec<u8>, String> {
    if !data.starts_with(&[0x1f, 0x8b]) {
        return Ok(data.to_vec());
    }
    let mut decoder = GzDecoder::new(data);
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|err| format!("failed to decompress gzip {label}: {err}"))?;
    Ok(decoded)
}

pub(super) fn looks_like_arm64_image(data: &[u8]) -> bool {
    if data.len() < 64 {
        return false;
    }
    let magic = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
    magic == ARM64_KERNEL_MAGIC
}
