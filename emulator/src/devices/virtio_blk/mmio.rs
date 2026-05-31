mod config;

use super::*;
use config::{mask_read, read_config_u64};

impl VirtioBlk {
    pub fn allocated_storage_bytes(&self) -> u64 {
        self.storage.allocated_bytes()
    }

    pub fn storage_generation(&self) -> u64 {
        self.storage.generation()
    }

    pub fn sparse_disk_size_bytes(&self) -> u64 {
        self.storage.capacity_bytes()
    }

    pub fn snapshot_sparse_disk(&self) -> Result<Vec<u8>, String> {
        self.storage.snapshot()
    }

    pub fn restore_sparse_disk(&mut self, snapshot: &[u8]) -> Result<(), String> {
        self.storage.restore(snapshot)?;
        self.reset_queue();
        Ok(())
    }

    pub fn read(&self, offset: u64, size: u8) -> Option<u64> {
        let val = match offset {
            0x000 => VIRTIO_MMIO_MAGIC,
            0x004 => VIRTIO_MMIO_VERSION_2,
            0x008 => VIRTIO_DEVICE_ID_BLOCK,
            0x00c => VIRTIO_VENDOR_WEBBOXVM,
            0x010 => self.selected_device_features(),
            0x034 => QUEUE_NUM_MAX as u64,
            0x044 => self.queue_ready as u64,
            0x060 => self.interrupt_status as u64,
            0x070 => self.status as u64,
            0x080 => self.queue_desc as u32 as u64,
            0x084 => self.queue_desc >> 32,
            0x090 => self.queue_driver as u32 as u64,
            0x094 => self.queue_driver >> 32,
            0x0a0 => self.queue_device as u32 as u64,
            0x0a4 => self.queue_device >> 32,
            0x0fc => 0,
            0x100..=0x107 => return read_config_u64(self.capacity_sectors(), offset - 0x100, size),
            _ => 0,
        };
        mask_read(val, size)
    }

    pub fn write(&mut self, mem: &mut PhysicalMemory, offset: u64, value: u64, _size: u8) -> bool {
        match offset {
            0x014 => self.device_features_sel = value as u32,
            0x020 => {
                let _ = value;
            }
            0x024 => self.driver_features_sel = value as u32,
            0x030 => self.queue_sel = value as u32,
            0x038 => self.queue_num = (value as u16).min(QUEUE_NUM_MAX),
            0x044 => {
                self.queue_ready = value & 1 != 0;
                if !self.queue_ready {
                    self.last_avail_idx = 0;
                }
            }
            0x050 => return self.process_queue(mem),
            0x060 => self.interrupt_status = value as u32,
            0x064 => self.interrupt_status &= !(value as u32),
            0x070 => {
                self.status = value as u32;
                if value == 0 {
                    self.reset_queue();
                }
            }
            0x080 => self.queue_desc = (self.queue_desc & !0xffff_ffff) | (value as u32 as u64),
            0x084 => {
                self.queue_desc = (self.queue_desc & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            0x090 => self.queue_driver = (self.queue_driver & !0xffff_ffff) | (value as u32 as u64),
            0x094 => {
                self.queue_driver =
                    (self.queue_driver & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            0x0a0 => self.queue_device = (self.queue_device & !0xffff_ffff) | (value as u32 as u64),
            0x0a4 => {
                self.queue_device =
                    (self.queue_device & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            _ => {}
        }
        false
    }

    fn selected_device_features(&self) -> u64 {
        let features = VIRTIO_F_VERSION_1 | self.storage.feature_bits();
        match self.device_features_sel {
            0 => features & 0xffff_ffff,
            1 => features >> 32,
            _ => 0,
        }
    }

    fn capacity_sectors(&self) -> u64 {
        self.storage.capacity_sectors()
    }

    pub(super) fn reset_queue(&mut self) {
        self.queue_ready = false;
        self.queue_num = 0;
        self.queue_desc = 0;
        self.queue_driver = 0;
        self.queue_device = 0;
        self.last_avail_idx = 0;
        self.interrupt_status = 0;
    }
}
