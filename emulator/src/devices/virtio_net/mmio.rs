use super::*;

impl VirtioNet {
    pub fn read(&self, offset: u64, size: u8) -> Option<u64> {
        let val = match offset {
            0x000 => VIRTIO_MMIO_MAGIC,
            0x004 => VIRTIO_MMIO_VERSION_2,
            0x008 => VIRTIO_DEVICE_ID_NET,
            0x00c => VIRTIO_VENDOR_WEBBOXVM,
            0x010 => self.selected_device_features(),
            0x034 => self.selected_queue().map_or(0, |_| QUEUE_NUM_MAX as u64),
            0x044 => self.selected_queue().map_or(0, |queue| queue.ready as u64),
            0x060 => self.interrupt_status as u64,
            0x070 => self.status as u64,
            0x080 => self
                .selected_queue()
                .map_or(0, |queue| queue.desc as u32 as u64),
            0x084 => self.selected_queue().map_or(0, |queue| queue.desc >> 32),
            0x090 => self
                .selected_queue()
                .map_or(0, |queue| queue.driver as u32 as u64),
            0x094 => self.selected_queue().map_or(0, |queue| queue.driver >> 32),
            0x0a0 => self
                .selected_queue()
                .map_or(0, |queue| queue.device as u32 as u64),
            0x0a4 => self.selected_queue().map_or(0, |queue| queue.device >> 32),
            0x0fc => 0,
            0x100..=0x107 => return self.read_config(offset - 0x100, size),
            _ => 0,
        };
        mask_read(val, size)
    }

    pub fn write(&mut self, mem: &mut PhysicalMemory, offset: u64, value: u64, _size: u8) -> bool {
        match offset {
            0x014 => self.device_features_sel = value as u32,
            0x020 => {}
            0x024 => self.driver_features_sel = value as u32,
            0x030 => self.queue_sel = value as u32,
            0x038 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.num = (value as u16).min(QUEUE_NUM_MAX);
                }
            }
            0x044 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.ready = value & 1 != 0;
                    if !queue.ready {
                        queue.last_avail_idx = 0;
                    }
                }
            }
            0x050 => return self.notify_queue(mem, value as u32),
            0x060 => self.interrupt_status = value as u32,
            0x064 => self.interrupt_status &= !(value as u32),
            0x070 => {
                self.status = value as u32;
                if value == 0 {
                    self.reset();
                }
            }
            0x080 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.desc = low32(queue.desc, value);
                }
            }
            0x084 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.desc = high32(queue.desc, value);
                }
            }
            0x090 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.driver = low32(queue.driver, value);
                }
            }
            0x094 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.driver = high32(queue.driver, value);
                }
            }
            0x0a0 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.device = low32(queue.device, value);
                }
            }
            0x0a4 => {
                if let Some(queue) = self.selected_queue_mut() {
                    queue.device = high32(queue.device, value);
                }
            }
            _ => {}
        }
        false
    }

    fn selected_device_features(&self) -> u64 {
        let features = VIRTIO_F_VERSION_1 | VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS;
        match self.device_features_sel {
            0 => features & 0xffff_ffff,
            1 => features >> 32,
            _ => 0,
        }
    }

    fn read_config(&self, offset: u64, size: u8) -> Option<u64> {
        let mut config = [0u8; 8];
        config[..6].copy_from_slice(&self.mac);
        config[6..8].copy_from_slice(&1u16.to_le_bytes());
        let len = access_len(size)?;
        let start = offset as usize;
        let end = start.checked_add(len)?;
        if end > config.len() {
            return None;
        }
        let mut bytes = [0u8; 8];
        bytes[..len].copy_from_slice(&config[start..end]);
        Some(u64::from_le_bytes(bytes))
    }
}

fn low32(current: u64, value: u64) -> u64 {
    (current & !0xffff_ffff) | (value as u32 as u64)
}

fn high32(current: u64, value: u64) -> u64 {
    (current & 0xffff_ffff) | ((value as u32 as u64) << 32)
}

fn mask_read(value: u64, size: u8) -> Option<u64> {
    Some(match size {
        1 => value & 0xff,
        2 => value & 0xffff,
        4 => value & 0xffff_ffff,
        8 => value,
        _ => return None,
    })
}

fn access_len(size: u8) -> Option<usize> {
    match size {
        1 | 2 | 4 | 8 => Some(size as usize),
        _ => None,
    }
}
