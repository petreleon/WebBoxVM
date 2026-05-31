use super::BootContext;
use crate::loader::iso::load_iso_boot_image;

impl BootContext {
    pub fn new_from_iso(iso_image: &[u8], num_cores: usize) -> Result<Self, String> {
        let boot = load_iso_boot_image(iso_image)?;
        let mut ctx = Self::new_with_initrd_and_bootargs(
            &boot.kernel,
            num_cores,
            &boot.initrd,
            &boot.bootargs,
        )?;
        ctx.attach_virtio_block(iso_image);
        Ok(ctx)
    }

    pub fn new_from_iso_owned(iso_image: Vec<u8>, num_cores: usize) -> Result<Self, String> {
        let boot = load_iso_boot_image(&iso_image)?;
        let mut ctx = Self::new_with_initrd_and_bootargs(
            &boot.kernel,
            num_cores,
            &boot.initrd,
            &boot.bootargs,
        )?;
        ctx.attach_virtio_block_owned(iso_image);
        Ok(ctx)
    }

    pub fn attach_virtio_block(&mut self, image: &[u8]) {
        self.machine.bus.virtio_blk.set_image(image);
    }

    pub fn attach_virtio_block_owned(&mut self, image: Vec<u8>) {
        self.machine.bus.virtio_blk.set_image_owned(image);
    }
}
