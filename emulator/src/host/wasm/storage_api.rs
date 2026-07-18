use super::*;
use wasm_bindgen::prelude::*;

impl Emulator {
    fn install_disk_allocated_bytes_unchecked(&self) -> u64 {
        if let Some(ref boot) = self.boot {
            boot.install_disk_allocated_bytes()
        } else {
            self.machine.bus.virtio_disk.allocated_storage_bytes()
        }
    }
}

#[wasm_bindgen]
impl Emulator {
    /// Number of allocated guest memory pages.
    pub fn allocated_pages(&self) -> usize {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.allocated_pages()
        } else {
            self.machine.bus.mem.allocated_pages()
        }
    }

    /// Bytes allocated by the sparse writable install disk.
    pub fn install_disk_allocated_bytes(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.install_disk_allocated_bytes_unchecked()
    }

    /// Virtual size of the sparse writable install disk.
    pub fn install_disk_size_bytes(&self) -> u64 {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.install_disk_size_bytes()
        } else {
            self.machine.bus.virtio_disk.sparse_disk_size_bytes()
        }
    }

    /// Monotonic generation changed by guest writes or snapshot restore.
    pub fn install_disk_generation(&self) -> u64 {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.install_disk_generation()
        } else {
            self.machine.bus.virtio_disk.storage_generation()
        }
    }

    /// Export a compact sparse-disk snapshot for browser persistence.
    pub fn install_disk_snapshot(&self) -> Vec<u8> {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.install_disk_snapshot().unwrap_or_default()
        } else {
            self.machine
                .bus
                .virtio_disk
                .snapshot_sparse_disk()
                .unwrap_or_default()
        }
    }

    /// Restore the sparse install disk from a persisted browser snapshot.
    pub fn restore_install_disk(&mut self, snapshot: Vec<u8>) -> String {
        let _access = self.require_parallel_idle();
        let result = if let Some(ref mut boot) = self.boot {
            boot.restore_install_disk(&snapshot)
        } else {
            self.machine.bus.virtio_disk.restore_sparse_disk(&snapshot)
        };

        match result {
            Ok(()) => format!(
                "OK: restored persistent disk, {} allocated",
                self.install_disk_allocated_bytes_unchecked()
            ),
            Err(err) => format!("ERR: {err}"),
        }
    }
}
