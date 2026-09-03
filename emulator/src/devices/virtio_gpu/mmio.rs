use super::*;

impl VirtioGpu {
    pub fn read(&self, offset: u64, size: u8) -> Option<u64> {
        let value = match offset {
            0x000 => VIRTIO_MMIO_MAGIC,
            0x004 => VIRTIO_MMIO_VERSION_2,
            0x008 => VIRTIO_DEVICE_ID_GPU,
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
            // VirtIO MMIO requires an all-ones length for an unsupported
            // shared-memory selector. Linux uses this sentinel to avoid
            // treating address zero as a host-visible region.
            0x0b0 | 0x0b4 => u32::MAX as u64,
            0x0b8 | 0x0bc => 0,
            0x0fc | 0x100 | 0x104 => 0,
            0x108 => 1,
            0x10c => super::three_d::CAPSET_COUNT as u64,
            _ => 0,
        };
        mask_read(value, size)
    }

    pub fn write(
        &mut self,
        mem: &mut crate::memory::PhysicalMemory,
        offset: u64,
        value: u64,
        _size: u8,
    ) -> bool {
        match offset {
            0x014 => self.device_features_sel = value as u32,
            0x020 => self.write_driver_features(value as u32),
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
            0x060 => {}
            0x064 => self.interrupt_status &= !(value as u32),
            0x070 => self.write_status(value as u32),
            0x080 => set_low(
                &mut self.selected_queue_mut().map(|queue| &mut queue.desc),
                value,
            ),
            0x084 => set_high(
                &mut self.selected_queue_mut().map(|queue| &mut queue.desc),
                value,
            ),
            0x090 => set_low(
                &mut self.selected_queue_mut().map(|queue| &mut queue.driver),
                value,
            ),
            0x094 => set_high(
                &mut self.selected_queue_mut().map(|queue| &mut queue.driver),
                value,
            ),
            0x0a0 => set_low(
                &mut self.selected_queue_mut().map(|queue| &mut queue.device),
                value,
            ),
            0x0a4 => set_high(
                &mut self.selected_queue_mut().map(|queue| &mut queue.device),
                value,
            ),
            0x0ac => {}
            0x104 => {}
            _ => {}
        }
        false
    }
}

fn set_low(target: &mut Option<&mut u64>, value: u64) {
    if let Some(target) = target {
        **target = (**target & !0xffff_ffff) | value as u32 as u64;
    }
}

fn set_high(target: &mut Option<&mut u64>, value: u64) {
    if let Some(target) = target {
        **target = (**target & 0xffff_ffff) | ((value as u32 as u64) << 32);
    }
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
