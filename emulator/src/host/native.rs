use crate::boot::{BootContext, merge_bootargs};
use crate::images::iso::load_iso_boot_image;
use crate::runtime::RunBackend;
use std::fs;
use std::io;
use std::path::Path;

pub type NativeVm = BootContext;

pub struct NativeBoot {
    pub context: NativeVm,
    pub source: NativeBootSource,
    pub expected_initrd: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeBootSource {
    Kernel,
    Iso(NativeIsoInfo),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeIsoInfo {
    pub kernel_path: String,
    pub initrd_paths: Vec<String>,
    pub bootargs: String,
}

pub fn boot_from_path(path: &str, cores: usize) -> io::Result<NativeBoot> {
    let image = fs::read(path)?;
    boot_from_image(path, &image, cores, None)
}

pub fn boot_from_image(
    path_hint: &str,
    image: &[u8],
    cores: usize,
    extra_bootargs: Option<&str>,
) -> io::Result<NativeBoot> {
    if is_iso_path(path_hint) {
        boot_from_iso_image(image, cores, extra_bootargs)
    } else {
        let mut context = BootContext::new(image, cores).map_err(io_other)?;
        enable_native_threads(&mut context);
        Ok(NativeBoot {
            context,
            source: NativeBootSource::Kernel,
            expected_initrd: None,
        })
    }
}

pub fn read_iso_boot_info(path: &str) -> io::Result<(NativeIsoInfo, usize, usize)> {
    let image = fs::read(path)?;
    let boot = load_iso_boot_image(&image).map_err(io_other)?;
    let info = NativeIsoInfo {
        kernel_path: boot.kernel_path,
        initrd_paths: boot.initrd_paths,
        bootargs: boot.bootargs,
    };
    Ok((info, boot.kernel.len(), boot.initrd.len()))
}

pub fn is_iso_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("iso"))
}

fn boot_from_iso_image(
    image: &[u8],
    cores: usize,
    extra_bootargs: Option<&str>,
) -> io::Result<NativeBoot> {
    let boot = load_iso_boot_image(image).map_err(io_other)?;
    let bootargs = merge_bootargs(&boot.bootargs, extra_bootargs.unwrap_or(""));
    let expected_initrd = boot.initrd.clone();
    let mut context =
        BootContext::new_with_initrd_and_bootargs(&boot.kernel, cores, &boot.initrd, &bootargs)
            .map_err(io_other)?;
    enable_native_threads(&mut context);
    context.attach_virtio_block(image);

    Ok(NativeBoot {
        context,
        source: NativeBootSource::Iso(NativeIsoInfo {
            kernel_path: boot.kernel_path,
            initrd_paths: boot.initrd_paths,
            bootargs,
        }),
        expected_initrd: Some(expected_initrd),
    })
}

fn io_other(err: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.into())
}

fn enable_native_threads(context: &mut BootContext) {
    if context.machine.cpus.len() > 1 {
        context.machine.set_run_backend(RunBackend::NativeThreads);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_path_detection_is_extension_based() {
        assert!(is_iso_path("debian.ISO"));
        assert!(!is_iso_path("Image"));
    }

    #[test]
    fn raw_image_boot_uses_kernel_source() {
        let boot = boot_from_image("Image", &[0; 64], 1, None).unwrap();

        assert_eq!(boot.source, NativeBootSource::Kernel);
        assert!(boot.expected_initrd.is_none());
        assert_eq!(boot.context.pc(), crate::constants::KERNEL_LOAD_ADDR);
    }
}
