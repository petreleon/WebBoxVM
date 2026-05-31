//! ISO9660 boot media support.
//!
//! This module extracts an ARM64 Linux kernel and initrd from a bootable ISO.
//! It is intentionally not a virtual CD-ROM device yet; the boot path passes
//! the extracted kernel/initrd to the existing serial Linux boot flow.

mod filesystem;
mod grub;

use crate::constants::ARM64_KERNEL_MAGIC;
use flate2::read::GzDecoder;
use std::io::Read;

use filesystem::IsoFs;
use grub::{parse_grub_boot_spec, BootSpec};

const DEFAULT_ISO_BOOTARGS: &str =
    "earlycon=pl011,0x09000000 console=ttyAMA0,115200n8 loglevel=7";

const GRUB_CONFIG_CANDIDATES: &[&str] = &[
    "/boot/grub/grub.cfg",
    "/efi/boot/grub.cfg",
    "/grub/grub.cfg",
];

const FALLBACK_PAIRS: &[(&str, &str)] = &[
    ("/casper/vmlinuz", "/casper/initrd"),
    ("/casper/vmlinuz", "/casper/initrd.gz"),
    ("/casper/vmlinuz.efi", "/casper/initrd"),
    ("/install.a64/vmlinuz", "/install.a64/initrd.gz"),
    ("/install/vmlinuz", "/install/initrd.gz"),
    ("/images/pxeboot/vmlinuz", "/images/pxeboot/initrd.img"),
    ("/boot/vmlinuz", "/boot/initrd.img"),
    ("/vmlinuz", "/initrd"),
    ("/vmlinuz", "/initrd.gz"),
    ("/image", "/initrd"),
    ("/image.gz", "/initrd.gz"),
];

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
        initrd.extend_from_slice(fs.read_file(path)?);
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

fn find_boot_spec(fs: &IsoFs<'_>) -> Result<BootSpec, String> {
    for cfg_path in GRUB_CONFIG_CANDIDATES {
        if let Ok(config) = fs.read_text_file(cfg_path) {
            if let Some(spec) = parse_grub_boot_spec(&config, |path| fs.exists(path)) {
                return Ok(spec);
            }
        }
    }

    for (kernel_path, initrd_path) in FALLBACK_PAIRS {
        if fs.exists(kernel_path) && fs.exists(initrd_path) {
            return Ok(BootSpec {
                kernel_path: (*kernel_path).to_string(),
                initrd_paths: vec![(*initrd_path).to_string()],
                bootargs: DEFAULT_ISO_BOOTARGS.to_string(),
            });
        }
    }

    Err("no supported ARM64 Linux kernel/initrd pair found in ISO".to_string())
}

fn prepare_kernel_image(data: &[u8]) -> Result<Vec<u8>, String> {
    let kernel = if data.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = GzDecoder::new(data);
        let mut decoded = Vec::new();
        decoder
            .read_to_end(&mut decoded)
            .map_err(|err| format!("failed to decompress gzip kernel: {err}"))?;
        decoded
    } else {
        data.to_vec()
    };

    if !looks_like_arm64_image(&kernel) {
        return Err(
            "ISO kernel is not an uncompressed ARM64 Linux Image; x86 ISOs are not supported"
                .to_string(),
        );
    }

    Ok(kernel)
}

fn looks_like_arm64_image(data: &[u8]) -> bool {
    if data.len() < 64 {
        return false;
    }
    let magic = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
    magic == ARM64_KERNEL_MAGIC
}

fn ensure_serial_bootargs(args: &str) -> String {
    let mut out = args.trim().to_string();
    if out.is_empty() {
        out.push_str(DEFAULT_ISO_BOOTARGS);
        return out;
    }
    if !out.contains("earlycon=") {
        out.push_str(" earlycon=pl011,0x09000000");
    }
    if !out.contains("console=ttyAMA") {
        out.push_str(" console=ttyAMA0,115200n8");
    }
    if !out.contains("loglevel=") {
        out.push_str(" loglevel=7");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm64_image_stub() -> Vec<u8> {
        let mut image = vec![0u8; 64];
        image[56..60].copy_from_slice(&ARM64_KERNEL_MAGIC.to_le_bytes());
        image
    }

    #[test]
    fn rejects_non_arm64_kernel() {
        let err = prepare_kernel_image(b"not a kernel").unwrap_err();
        assert!(err.contains("ARM64"));
    }

    #[test]
    fn appends_serial_bootargs_when_missing() {
        let args = ensure_serial_bootargs("root=/dev/ram0 quiet");
        assert!(args.contains("root=/dev/ram0"));
        assert!(args.contains("earlycon=pl011,0x09000000"));
        assert!(args.contains("console=ttyAMA0,115200n8"));
        assert!(args.contains("loglevel=7"));
    }

    #[test]
    fn extracts_kernel_and_initrd_from_minimal_iso() {
        let iso = filesystem::tests::minimal_iso_with_files(&[
            ("/vmlinuz", &arm64_image_stub()),
            ("/initrd", b"initrd bytes"),
        ]);

        let boot = load_iso_boot_image(&iso).unwrap();
        assert_eq!(boot.kernel.len(), 64);
        assert_eq!(boot.initrd, b"initrd bytes");
        assert_eq!(boot.kernel_path, "/vmlinuz");
        assert_eq!(boot.initrd_paths, vec!["/initrd"]);
        assert!(boot.bootargs.contains("console=ttyAMA0"));
    }
}
