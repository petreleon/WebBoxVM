use super::state::{affinity_value, route_affinity};
use super::*;

const GICD_IROUTER_BASE: u64 = 0x6000;
const IROUTER_IRM: u64 = 1 << 31;
const IROUTER_WRITABLE: u64 = 0x0000_00ff_80ff_ffff;

impl Gicv3 {
    /// Return the MPIDR/IROUTER-shaped affinity assigned to a CPU.
    pub fn cpu_affinity(&self, cpu_id: usize) -> Option<u64> {
        (cpu_id < self.cpu_count()).then(|| route_affinity(cpu_id))
    }

    pub fn interrupt_route(&self, int_id: u32) -> Option<u64> {
        if !(SPI_FIRST..MAX_INTERRUPTS as u32).contains(&int_id) {
            return None;
        }
        Some(self.irouter[int_id as usize])
    }

    pub fn set_interrupt_route(&mut self, int_id: u32, route: u64) -> bool {
        if !(SPI_FIRST..MAX_INTERRUPTS as u32).contains(&int_id) {
            return false;
        }
        self.irouter[int_id as usize] = route & IROUTER_WRITABLE;
        true
    }

    /// Decode an ICC_SGI1R_EL1 value and pend its SGI on matching CPUs.
    ///
    /// Target-list routing supports all sixteen Aff0 bits (and RangeSelector
    /// groups); IRM broadcasts to every participating CPU except the sender.
    /// The return value is the number of redistributors targeted.
    pub fn route_sgi1r(&mut self, sender_cpu: usize, value: u64) -> usize {
        let int_id = ((value >> 24) & 0xf) as u32;
        let irm = value & (1 << 40) != 0;
        let target_list = value as u16;
        let aff1 = ((value >> 16) & 0xff) as u32;
        let aff2 = ((value >> 32) & 0xff) as u32;
        let range_selector = ((value >> 44) & 0xf) as u32;
        let aff3 = ((value >> 48) & 0xff) as u32;
        let aff0_base = range_selector * 16;
        let pending_bit = 1u32 << int_id;
        let mut targets = 0;

        for (cpu_id, redistributor) in self.redistributors.iter_mut().enumerate() {
            let matches = if irm {
                cpu_id != sender_cpu
            } else {
                let affinity = affinity_value(cpu_id);
                let cpu_aff0 = affinity & 0xff;
                let same_cluster = (affinity >> 8) & 0xff == aff1
                    && (affinity >> 16) & 0xff == aff2
                    && (affinity >> 24) & 0xff == aff3;
                let aff0_offset = cpu_aff0.wrapping_sub(aff0_base);
                same_cluster && aff0_offset < 16 && target_list & (1u16 << aff0_offset) != 0
            };
            if matches {
                redistributor.pending |= pending_bit;
                targets += 1;
            }
        }
        targets
    }

    pub(super) fn spi_targets_cpu(&self, int_id: u32, cpu_id: usize) -> bool {
        let Some(route) = self.interrupt_route(int_id) else {
            return false;
        };
        if cpu_id >= self.cpu_count() {
            return false;
        }
        if route & IROUTER_IRM != 0 {
            // The model has no load-balancing state, so IRM deterministically
            // selects the lowest participating affinity.
            return cpu_id == 0;
        }
        route & !IROUTER_IRM == route_affinity(cpu_id)
    }

    pub(super) fn read_irouter_mmio(&self, offset: u64, size: u8) -> u64 {
        let (int_id, byte_offset) = irouter_location(offset);
        let Some(route) = self.interrupt_route(int_id) else {
            return 0;
        };
        extract_register_bytes(route, byte_offset, size)
    }

    pub(super) fn write_irouter_mmio(&mut self, offset: u64, value: u64, size: u8) {
        let (int_id, byte_offset) = irouter_location(offset);
        let Some(old_route) = self.interrupt_route(int_id) else {
            return;
        };
        let Some((mask, shift)) = register_byte_mask(byte_offset, size) else {
            return;
        };
        let merged = (old_route & !(mask << shift)) | ((value & mask) << shift);
        self.set_interrupt_route(int_id, merged);
    }
}

fn irouter_location(offset: u64) -> (u32, u8) {
    let relative = offset - GICD_IROUTER_BASE;
    ((relative / 8) as u32, (relative % 8) as u8)
}

fn extract_register_bytes(value: u64, byte_offset: u8, size: u8) -> u64 {
    let Some((mask, shift)) = register_byte_mask(byte_offset, size) else {
        return 0;
    };
    (value >> shift) & mask
}

fn register_byte_mask(byte_offset: u8, size: u8) -> Option<(u64, u32)> {
    if size == 0 || size > 8 || byte_offset as u16 + size as u16 > 8 {
        return None;
    }
    let mask = if size == 8 {
        u64::MAX
    } else {
        (1u64 << (size as u32 * 8)) - 1
    };
    Some((mask, byte_offset as u32 * 8))
}
