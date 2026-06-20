use super::BootPlan;
use crate::images::iso::load_iso_boot_image;

impl BootPlan {
    pub fn new_from_iso(iso_image: &[u8], num_cores: usize) -> Result<Self, String> {
        let boot = load_iso_boot_image(iso_image)?;
        Ok(Self::new_with_initrd_and_bootargs(
            &boot.kernel,
            num_cores,
            &boot.initrd,
            &boot.bootargs,
        )?
        .with_boot_media(iso_image.to_vec()))
    }

    pub fn new_from_iso_owned(iso_image: Vec<u8>, num_cores: usize) -> Result<Self, String> {
        let boot = load_iso_boot_image(&iso_image)?;
        Ok(Self::new_with_initrd_and_bootargs(
            &boot.kernel,
            num_cores,
            &boot.initrd,
            &boot.bootargs,
        )?
        .with_boot_media(iso_image))
    }
}
