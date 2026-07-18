use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Load and configure a Linux kernel Image for boot.
    /// `kernel_image`: raw bytes of an ARM64 Linux Image
    /// `num_cores`: number of ARM64 cores to emulate
    pub fn boot_kernel(&mut self, kernel_image: Vec<u8>, num_cores: usize) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new(&kernel_image, num_cores) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!("OK: kernel loaded, {} cores ready", cores)
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load and configure a Linux kernel Image with a caller-provided initrd.
    pub fn boot_kernel_with_initrd(
        &mut self,
        kernel_image: Vec<u8>,
        initrd: Vec<u8>,
        num_cores: usize,
    ) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new_with_initrd(&kernel_image, num_cores, &initrd) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!(
                    "OK: kernel loaded with custom initrd, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load and configure a Linux kernel Image with a caller-provided BusyBox binary.
    pub fn boot_kernel_with_busybox(
        &mut self,
        kernel_image: Vec<u8>,
        busybox: Vec<u8>,
        num_cores: usize,
    ) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new_with_busybox(&kernel_image, num_cores, &busybox) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!(
                    "OK: kernel loaded with BusyBox initrd, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load an ARM64 Linux ISO by extracting its kernel/initrd boot pair.
    pub fn boot_iso(&mut self, iso_image: Vec<u8>, num_cores: usize) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new_from_iso_owned(iso_image, num_cores) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!("OK: ISO kernel/initrd loaded, {} cores ready", cores)
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Load an ARM64 Linux ISO and configure a writable sparse install disk.
    pub fn boot_iso_with_disk(
        &mut self,
        iso_image: Vec<u8>,
        num_cores: usize,
        disk_size_bytes: u64,
    ) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new_from_iso_owned(iso_image, num_cores) {
            Ok(mut ctx) => {
                ctx.set_install_disk_size(disk_size_bytes);
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!(
                    "OK: ISO kernel/initrd loaded with writable disk, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }

    /// Boot the installed Linux system from a persisted sparse disk snapshot.
    pub fn boot_installed_disk(&mut self, disk_snapshot: Vec<u8>, num_cores: usize) -> String {
        self.boot_installed_disk_with_extra_bootargs(disk_snapshot, num_cores, String::new())
    }

    /// Boot the installed Linux system with extra kernel command-line args.
    pub fn boot_installed_disk_with_extra_bootargs(
        &mut self,
        disk_snapshot: Vec<u8>,
        num_cores: usize,
        extra_bootargs: String,
    ) -> String {
        let Ok(_access) = self.try_parallel_idle() else {
            return "ERR: parallel run must be finished before replacing the VM".into();
        };
        match BootContext::new_from_install_disk_snapshot_with_extra_bootargs(
            disk_snapshot,
            num_cores,
            &extra_bootargs,
        ) {
            Ok(ctx) => {
                let cores = ctx.machine.cpus.len();
                self.boot = Some(Box::new(ctx));
                format!(
                    "OK: installed disk kernel/initrd loaded, {} cores ready",
                    cores
                )
            }
            Err(e) => format!("ERR: {}", e),
        }
    }
}
