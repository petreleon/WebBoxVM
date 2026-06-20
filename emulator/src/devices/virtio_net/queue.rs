use super::*;

impl VirtioNet {
    pub(super) fn notify_queue(&mut self, mem: &mut PhysicalMemory, queue: u32) -> bool {
        match queue as usize {
            QUEUE_RX => self.process_rx(mem),
            QUEUE_TX => self.process_tx(mem),
            _ => false,
        }
    }

    pub(super) fn process_tx(&mut self, mem: &mut PhysicalMemory) -> bool {
        let Some(avail_idx) = self.avail_idx(mem, QUEUE_TX) else {
            return false;
        };
        let mut completed = false;
        while self.queues[QUEUE_TX].last_avail_idx != avail_idx {
            let Some(head) = self.avail_head(mem, QUEUE_TX) else {
                break;
            };
            let frame = self
                .descriptor_chain(mem, QUEUE_TX, head)
                .and_then(|chain| read_tx_frame(mem, &chain));
            if let Some(frame) = frame {
                self.tx_frames.push_back(frame);
                self.tx_packets = self.tx_packets.wrapping_add(1);
            }
            self.push_used(mem, QUEUE_TX, head as u32, 0);
            self.queues[QUEUE_TX].last_avail_idx =
                self.queues[QUEUE_TX].last_avail_idx.wrapping_add(1);
            completed = true;
        }
        self.finish_queue(completed)
    }

    pub(super) fn process_rx(&mut self, mem: &mut PhysicalMemory) -> bool {
        let Some(avail_idx) = self.avail_idx(mem, QUEUE_RX) else {
            return false;
        };
        let mut completed = false;
        while self.queues[QUEUE_RX].last_avail_idx != avail_idx && !self.rx_frames.is_empty() {
            let Some(head) = self.avail_head(mem, QUEUE_RX) else {
                break;
            };
            let Some(chain) = self.descriptor_chain(mem, QUEUE_RX, head) else {
                break;
            };
            let frame = self.rx_frames.front().expect("rx frame checked");
            let written = write_rx_frame(mem, &chain, frame).unwrap_or(0);
            if written > 0 {
                self.rx_frames.pop_front();
                self.rx_packets = self.rx_packets.wrapping_add(1);
            }
            self.push_used(mem, QUEUE_RX, head as u32, written);
            self.queues[QUEUE_RX].last_avail_idx =
                self.queues[QUEUE_RX].last_avail_idx.wrapping_add(1);
            completed = true;
        }
        self.finish_queue(completed)
    }

    fn finish_queue(&mut self, completed: bool) -> bool {
        if completed {
            self.interrupt_status |= 1;
        }
        completed
    }

    fn avail_idx(&self, mem: &PhysicalMemory, queue: usize) -> Option<u16> {
        let queue = self.queues.get(queue)?;
        if !queue.ready || queue.num == 0 || queue.desc == 0 {
            return None;
        }
        mem.read(queue.driver + 2, 2).map(|value| value as u16)
    }

    fn avail_head(&self, mem: &PhysicalMemory, queue_index: usize) -> Option<u16> {
        let queue = self.queues.get(queue_index)?;
        let slot = queue.last_avail_idx % queue.num;
        mem.read(queue.driver + 4 + slot as u64 * 2, 2)
            .map(|value| value as u16)
    }

    fn descriptor_chain(
        &self,
        mem: &PhysicalMemory,
        queue_index: usize,
        head: u16,
    ) -> Option<Vec<Descriptor>> {
        let queue = self.queues.get(queue_index)?;
        let mut chain = Vec::new();
        let mut desc = self.read_desc(mem, queue_index, head)?;
        for _ in 0..queue.num {
            chain.push(desc);
            if desc.flags & VIRTQ_DESC_F_NEXT == 0 {
                return Some(chain);
            }
            desc = self.read_desc(mem, queue_index, desc.next)?;
        }
        None
    }

    fn read_desc(&self, mem: &PhysicalMemory, queue: usize, index: u16) -> Option<Descriptor> {
        let queue = self.queues.get(queue)?;
        if index >= queue.num {
            return None;
        }
        let base = queue.desc + index as u64 * 16;
        Some(Descriptor {
            addr: mem.read(base, 8)?,
            len: mem.read(base + 8, 4)? as u32,
            flags: mem.read(base + 12, 2)? as u16,
            next: mem.read(base + 14, 2)? as u16,
        })
    }

    fn push_used(&mut self, mem: &mut PhysicalMemory, queue_index: usize, id: u32, len: u32) {
        let queue = self.queues[queue_index];
        let used_idx = mem.read(queue.device + 2, 2).unwrap_or(0) as u16;
        let elem = queue.device + 4 + (used_idx % queue.num) as u64 * 8;
        let _ = mem.write(elem, 4, id as u64);
        let _ = mem.write(elem + 4, 4, len as u64);
        let _ = mem.write(queue.device + 2, 2, used_idx.wrapping_add(1) as u64);
    }
}

fn read_tx_frame(mem: &PhysicalMemory, chain: &[Descriptor]) -> Option<Vec<u8>> {
    let mut packet = Vec::new();
    for desc in chain {
        if desc.flags & VIRTQ_DESC_F_WRITE != 0 {
            continue;
        }
        let mut bytes = vec![0; desc.len as usize];
        mem.read_bytes(desc.addr, &mut bytes)?;
        packet.extend_from_slice(&bytes);
    }
    if packet.len() <= VIRTIO_NET_HDR_LEN {
        return None;
    }
    Some(packet[VIRTIO_NET_HDR_LEN..].to_vec())
}

fn write_rx_frame(mem: &mut PhysicalMemory, chain: &[Descriptor], frame: &[u8]) -> Option<u32> {
    let mut packet = vec![0; VIRTIO_NET_HDR_LEN];
    packet.extend_from_slice(frame);
    let capacity = chain.iter().try_fold(0usize, |sum, desc| {
        (desc.flags & VIRTQ_DESC_F_WRITE != 0).then_some(sum + desc.len as usize)
    })?;
    if capacity < packet.len() {
        return Some(0);
    }
    let mut done = 0usize;
    for desc in chain {
        let count = (desc.len as usize).min(packet.len() - done);
        if count > 0 {
            mem.write_bytes(desc.addr, &packet[done..done + count])?;
        }
        done += count;
        if done == packet.len() {
            break;
        }
    }
    Some(done as u32)
}
