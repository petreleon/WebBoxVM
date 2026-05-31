use crate::arm64::Machine;
use crate::constants::*;
use crate::dtb::{build_dtb, load_dtb};
use crate::initrd::load_initrd;

mod initrd;
mod iso;
mod runtime;
#[cfg(test)]
mod tests;

pub use self::initrd::{
    DEFAULT_BOOTARGS, DEFAULT_BUSYBOX_AARCH64, build_busybox_initrd, build_default_initrd,
};

/// Holds everything needed to boot and run a Linux kernel.
pub struct BootContext {
    pub machine: Machine,
    pub dtb_addr: u64,
}

impl BootContext {
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

        let mut machine = Machine::new(num_cores);

        machine
            .bus
            .mem
            .write_bytes(KERNEL_LOAD_ADDR, kernel_image)
            .ok_or_else(|| "kernel image does not fit in guest RAM".to_string())?;

        // Standard ARM64 Linux boot protocol:
        // X0 = physical address of DTB, X1-X3 = 0, MMU off
        let cpu0 = &mut machine.cpus[0];
        cpu0.regs.set_x(0, DTB_BASE);
        cpu0.regs.set_x(1, 0);
        cpu0.regs.set_x(2, 0);
        cpu0.regs.set_x(3, 0);
        cpu0.pstate = cpu0.pstate.with_el(1).with_irq_masked(true);
        cpu0.sys.sctlr_el1 = 0; // MMU disabled — kernel's head.S enables it
        // Jump to ARM64 Image header (code0+cod1 branch to primary_entry)
        cpu0.regs.pc = KERNEL_LOAD_ADDR;

        // Build DTB and load the initrd into guest RAM.
        let dtb = build_dtb(
            RAM_BASE,
            RAM_SIZE,
            Some(INITRD_BASE),
            Some(initrd_end),
            Some(bootargs),
        );
        load_initrd(&mut machine.bus, INITRD_BASE, initrd);
        load_dtb(&mut machine.bus, DTB_BASE, &dtb);

        Ok(BootContext {
            machine,
            dtb_addr: DTB_BASE,
        })
    }

    pub fn set_install_disk_size(&mut self, size_bytes: u64) {
        self.machine.bus.virtio_disk.set_sparse_disk(size_bytes);
    }
}
