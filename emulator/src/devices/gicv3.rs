//! GICv3 interrupt controller: minimal distributor + redistributor MMIO.
//! CPU interface is handled via system registers in system_regs.rs.

pub struct Gicv3 {
    // Distributor registers
    pub ctld: u64,       // GICD_CTLR (0x0000)
    pub typer: u64,      // GICD_TYPER (0x0008, RO)
    pub iidr: u32,       // GICD_IIDR (0x0018, RO)
    pub enable: [u32; 32], // GICD_ISENABLER / ICENABLER (0x0100-0x017C)
    pub pending: [u32; 32], // GICD_ISPENDR (0x0200)
    pub priority: [u8; 1024], // GICD_IPRIORITYR (0x0400)
    pub group: [u32; 32],  // GICD_IGROUPR / IGRPMODR

    // Redistributor registers (one core)
    pub rctlr: u64,       // GICR_CTLR
    pub rwaker: u64,      // GICR_WAKER
    pub rtyper: u64,      // GICR_TYPER (RO)
}

impl Gicv3 {
    pub fn new() -> Self {
        Self {
            ctld: 0,
            typer: 0, // ITLinesNumber = 0 (32 interrupts supported)
            iidr: 0x0201743B, // GICv3 r0, ARM implementation
            enable: [0; 32],
            pending: [0; 32],
            priority: [0; 1024],
            group: [0; 32],
            rctlr: 0,
            rwaker: 0,
            rtyper: 0, // ProcessorNumber = 0, affinity-based routing
        }
    }

    /// Handle GICD MMIO read
    pub fn gicd_read(&self, offset: u64, size: u8) -> Option<u64> {
        match offset {
            0x0000 => Some(self.ctld),
            0x0004 => Some(self.ctld >> 32),
            0x0008 => Some(self.typer),
            0x0018 => Some(self.iidr as u64),
            o if (0x0100..0x0180).contains(&o) => {
                let idx = ((o - 0x0100) / 4) as usize;
                if idx < 32 { Some(self.enable[idx] as u64) } else { Some(0) }
            }
            o if (0x0200..0x0280).contains(&o) => {
                let idx = ((o - 0x0200) / 4) as usize;
                if idx < 32 { Some(self.pending[idx] as u64) } else { Some(0) }
            }
            o if (0x0400..0x0800).contains(&o) => {
                let idx = (o - 0x0400) as usize;
                if idx < 1024 {
                    let mut val = 0u64;
                    for i in 0..4.min(size as usize) {
                        if idx + i < 1024 {
                            val |= (self.priority[idx + i] as u64) << (i * 8);
                        }
                    }
                    Some(val)
                } else { Some(0) }
            }
            o if (0x0800..0x0880).contains(&o) => {
                let idx = ((o - 0x0800) / 4) as usize;
                if idx < 32 { Some(self.group[idx] as u64) } else { Some(0) }
            }
            _ => Some(0),
        }
    }

    /// Handle GICD MMIO write
    pub fn gicd_write(&mut self, offset: u64, value: u64, size: u8) {
        match offset {
            0x0000 => self.ctld = value,
            o if (0x0100..0x0180).contains(&o) => {
                let idx = ((o - 0x0100) / 4) as usize;
                if idx < 32 { self.enable[idx] = value as u32; }
            }
            o if (0x0180..0x0200).contains(&o) => {
                let idx = ((o - 0x0180) / 4) as usize;
                if idx < 32 { self.enable[idx] &= !(value as u32); }
            }
            o if (0x0200..0x0280).contains(&o) => {
                let idx = ((o - 0x0200) / 4) as usize;
                if idx < 32 { self.pending[idx] = value as u32; }
            }
            o if (0x0280..0x0300).contains(&o) => {
                let idx = ((o - 0x0280) / 4) as usize;
                if idx < 32 { self.pending[idx] &= !(value as u32); }
            }
            o if (0x0400..0x0800).contains(&o) => {
                let idx = (o - 0x0400) as usize;
                for i in 0..(size as usize).min(4) {
                    if idx + i < 1024 {
                        self.priority[idx + i] = ((value >> (i * 8)) & 0xFF) as u8;
                    }
                }
            }
            o if (0x0800..0x0880).contains(&o) => {
                let idx = ((o - 0x0800) / 4) as usize;
                if idx < 32 { self.group[idx] = value as u32; }
            }
            _ => {}
        }
    }

    /// Handle GICR MMIO read
    pub fn gicr_read(&self, offset: u64, _size: u8) -> Option<u64> {
        match offset {
            0x0000 => Some(self.rctlr),
            0x0004 => Some((self.rctlr >> 32) as u64),
            0x0008 => Some(self.iidr as u64),
            0x0014 => Some(self.rwaker as u64),
            0x001C => Some((self.rwaker >> 32) as u64),
            0x0000..=0x001F if offset >= 0x0008 && offset < 0x0010 => Some(self.rtyper),
            _ => Some(0),
        }
    }

    /// Handle GICR MMIO write
    pub fn gicr_write(&mut self, offset: u64, value: u64, _size: u8) {
        match offset {
            0x0000 => self.rctlr = value,
            0x0014 => self.rwaker = value,
            _ => {}
        }
    }
}
