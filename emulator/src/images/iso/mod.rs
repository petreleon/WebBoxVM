//! ISO9660 boot media support.
//!
//! This module extracts an ARM64 Linux kernel and initrd from a bootable ISO.
//! It is intentionally not a virtual CD-ROM device yet; the boot path passes
//! the extracted kernel/initrd to the existing serial Linux boot flow.

mod boot_spec;
mod bootargs;
mod filesystem;
mod grub;
mod images;

use boot_spec::find_boot_spec;
use bootargs::ensure_serial_bootargs;
use filesystem::IsoFs;
use images::{prepare_initrd_image, prepare_kernel_image};

#[derive(Debug, Clone)]
pub struct IsoBootImage {
    pub kernel: Vec<u8>,
    pub initrd: Vec<u8>,
    pub bootargs: String,
    pub kernel_path: String,
    pub initrd_paths: Vec<String>,
}

/// Extract a bootable ARM64 kernel/initrd pair from an ISO9660 image.
pub fn load_iso_boot_image(data: &[u8]) -> Result<IsoBootImage, String> {
    let fs = IsoFs::parse(data)?;
    let spec = find_boot_spec(&fs)?;

    let kernel_raw = fs.read_file(&spec.kernel_path)?.to_vec();
    let kernel = prepare_kernel_image(&kernel_raw)?;

    let mut initrd = Vec::new();
    for path in &spec.initrd_paths {
        initrd.extend_from_slice(&prepare_initrd_image(fs.read_file(path)?)?);
    }
    if initrd.is_empty() {
        return Err("ISO boot initrd is empty".to_string());
    }

    Ok(IsoBootImage {
        kernel,
        initrd,
        bootargs: ensure_serial_bootargs(&spec.bootargs),
        kernel_path: spec.kernel_path,
        initrd_paths: spec.initrd_paths,
    })
}

#[cfg(test)]
mod tests;
