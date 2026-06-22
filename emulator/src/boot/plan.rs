use super::initrd::{DEFAULT_BOOTARGS, build_busybox_initrd, build_default_initrd};
use crate::constants::*;
use crate::dtb::build_dtb_with_boot_media_device;

/// Pure boot artifact bundle. It owns bytes and addresses, never a live VM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootPlan {
    pub num_cores: usize,
    pub kernel_image: Vec<u8>,
    pub initrd_image: Vec<u8>,
    pub dtb_image: Vec<u8>,
    pub bootargs: String,
    pub boot_media: Option<Vec<u8>>,
    pub entry: u64,
    pub dtb_addr: u64,
    pub initrd_addr: u64,
    pub initrd_end: u64,
}

impl BootPlan {
    pub fn new(kernel_image: &[u8], num_cores: usize) -> Result<Self, String> {
        Self::new_with_initrd(kernel_image, num_cores, &build_default_initrd())
    }

    pub fn new_with_busybox(
        kernel_image: &[u8],
        num_cores: usize,
        busybox: &[u8],
    ) -> Result<Self, String> {
        let initrd = build_busybox_initrd(busybox)?;
        Self::new_with_initrd(kernel_image, num_cores, &initrd)
    }

    pub fn new_with_initrd(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
    ) -> Result<Self, String> {
        Self::new_with_initrd_and_bootargs(kernel_image, num_cores, initrd, DEFAULT_BOOTARGS)
    }

    pub fn new_with_initrd_and_bootargs(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
        bootargs: &str,
    ) -> Result<Self, String> {
        Self::new_with_initrd_bootargs_and_media_device(
            kernel_image,
            num_cores,
            initrd,
            bootargs,
            true,
        )
    }

    pub(crate) fn new_installed_disk(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
        bootargs: &str,
    ) -> Result<Self, String> {
        Self::new_with_initrd_bootargs_and_media_device(
            kernel_image,
            num_cores,
            initrd,
            bootargs,
            false,
        )
    }

    fn new_with_initrd_bootargs_and_media_device(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
        bootargs: &str,
        advertise_boot_media: bool,
    ) -> Result<Self, String> {
        let initrd_end = validate_inputs(num_cores, initrd)?;
        let dtb_image = build_dtb_with_boot_media_device(
            RAM_BASE,
            RAM_SIZE,
            Some(INITRD_BASE),
            Some(initrd_end),
            Some(bootargs),
            advertise_boot_media,
        );

        Ok(Self {
            num_cores,
            kernel_image: kernel_image.to_vec(),
            initrd_image: initrd.to_vec(),
            dtb_image,
            bootargs: bootargs.to_string(),
            boot_media: None,
            entry: KERNEL_LOAD_ADDR,
            dtb_addr: DTB_BASE,
            initrd_addr: INITRD_BASE,
            initrd_end,
        })
    }

    pub(crate) fn with_boot_media(mut self, media: Vec<u8>) -> Self {
        self.boot_media = Some(media);
        self
    }
}

fn validate_inputs(num_cores: usize, initrd: &[u8]) -> Result<u64, String> {
    if num_cores == 0 {
        return Err("num_cores must be at least 1".to_string());
    }
    if initrd.is_empty() {
        return Err("initrd must not be empty".to_string());
    }

    let initrd_end = INITRD_BASE
        .checked_add(initrd.len() as u64)
        .ok_or_else(|| "initrd address overflow".to_string())?;
    if initrd_end >= DTB_BASE {
        return Err(format!(
            "initrd too large: end {initrd_end:#x} overlaps DTB at {DTB_BASE:#x}"
        ));
    }
    Ok(initrd_end)
}
