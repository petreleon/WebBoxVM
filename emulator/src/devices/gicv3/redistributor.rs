use super::state::redistributor_typer;
use super::*;

const SGI_FRAME_BASE: u64 = 0x1_0000;
const GICR_IGROUPR0: u64 = SGI_FRAME_BASE + 0x0080;
const GICR_ISENABLER0: u64 = SGI_FRAME_BASE + 0x0100;
const GICR_ICENABLER0: u64 = SGI_FRAME_BASE + 0x0180;
const GICR_ISPENDR0: u64 = SGI_FRAME_BASE + 0x0200;
const GICR_ICPENDR0: u64 = SGI_FRAME_BASE + 0x0280;
const GICR_IPRIORITYR0: u64 = SGI_FRAME_BASE + 0x0400;
const GICR_IPRIORITYR_END: u64 = GICR_IPRIORITYR0 + PRIVATE_INTERRUPTS as u64;

impl Gicv3 {
    pub fn redistributor_typer_for_cpu(&self, cpu_id: usize) -> Option<u64> {
        if cpu_id >= self.cpu_count() {
            return None;
        }
        Some(if cpu_id == 0 {
            self.rtyper
        } else {
            redistributor_typer(cpu_id, self.cpu_count())
        })
    }

    pub fn gicr_read(&self, offset: u64, size: u8) -> Option<u64> {
        let Some((cpu_id, local)) = self.redistributor_location(offset) else {
            return Some(0);
        };
        let redistributor = &self.redistributors[cpu_id];
        let value = match local {
            0x0000 => self.redistributor_ctlr(cpu_id),
            0x0004 => self.iidr as u64,
            0x0008 => self.redistributor_typer_for_cpu(cpu_id).unwrap_or(0),
            0x000C => self.redistributor_typer_for_cpu(cpu_id).unwrap_or(0) >> 32,
            0x0014 => self.redistributor_waker(cpu_id),
            0x001C => self.redistributor_waker(cpu_id) >> 32,
            0xFFE8 | 0x1_FFE8 => GIC_PIDR2_ARCH_GICV3,
            GICR_IGROUPR0 => redistributor.group as u64,
            GICR_ISENABLER0 | GICR_ICENABLER0 => redistributor.enable as u64,
            GICR_ISPENDR0 | GICR_ICPENDR0 => redistributor.pending as u64,
            o if (GICR_IPRIORITYR0..GICR_IPRIORITYR_END).contains(&o) => {
                self.read_private_priority(cpu_id, o, size)
            }
            _ => 0,
        };
        Some(value)
    }

    pub fn gicr_write(&mut self, offset: u64, value: u64, size: u8) {
        let Some((cpu_id, local)) = self.redistributor_location(offset) else {
            return;
        };
        match local {
            0x0000 => self.set_redistributor_ctlr(cpu_id, value),
            0x0014 => self.set_redistributor_waker(cpu_id, value),
            GICR_IGROUPR0 => self.redistributors[cpu_id].group = value as u32,
            GICR_ISENABLER0 => self.redistributors[cpu_id].enable |= value as u32,
            GICR_ICENABLER0 => self.redistributors[cpu_id].enable &= !(value as u32),
            GICR_ISPENDR0 => self.redistributors[cpu_id].pending |= value as u32,
            GICR_ICPENDR0 => self.redistributors[cpu_id].pending &= !(value as u32),
            o if (GICR_IPRIORITYR0..GICR_IPRIORITYR_END).contains(&o) => {
                self.write_private_priority(cpu_id, o, value, size);
            }
            _ => {}
        }
    }

    fn redistributor_location(&self, offset: u64) -> Option<(usize, u64)> {
        let cpu_id = (offset / GICR_FRAME_SIZE) as usize;
        (cpu_id < self.cpu_count()).then_some((cpu_id, offset % GICR_FRAME_SIZE))
    }

    fn redistributor_ctlr(&self, cpu_id: usize) -> u64 {
        if cpu_id == 0 {
            self.rctlr
        } else {
            self.redistributors[cpu_id].ctlr
        }
    }

    fn set_redistributor_ctlr(&mut self, cpu_id: usize, value: u64) {
        self.redistributors[cpu_id].ctlr = value;
        if cpu_id == 0 {
            self.rctlr = value;
        }
    }

    fn redistributor_waker(&self, cpu_id: usize) -> u64 {
        if cpu_id == 0 {
            self.rwaker
        } else {
            self.redistributors[cpu_id].waker
        }
    }

    fn set_redistributor_waker(&mut self, cpu_id: usize, value: u64) {
        self.redistributors[cpu_id].waker = value;
        if cpu_id == 0 {
            self.rwaker = value;
        }
    }

    fn read_private_priority(&self, cpu_id: usize, offset: u64, size: u8) -> u64 {
        let first = (offset - GICR_IPRIORITYR0) as usize;
        let mut value = 0;
        for byte in 0..(size as usize).min(8) {
            if first + byte < PRIVATE_INTERRUPTS {
                value |= (self.redistributors[cpu_id].priority[first + byte] as u64) << (byte * 8);
            }
        }
        value
    }

    fn write_private_priority(&mut self, cpu_id: usize, offset: u64, value: u64, size: u8) {
        let first = (offset - GICR_IPRIORITYR0) as usize;
        for byte in 0..(size as usize).min(8) {
            if first + byte < PRIVATE_INTERRUPTS {
                self.redistributors[cpu_id].priority[first + byte] =
                    ((value >> (byte * 8)) & 0xff) as u8;
            }
        }
    }
}
