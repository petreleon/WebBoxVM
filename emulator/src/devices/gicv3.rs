//! GICv3 Interrupt Controller — minimal distributor + redistributor MMIO emulation.
//!
//! The ARM Generic Interrupt Controller (GIC) manages hardware interrupts.
//! We only implement the bare minimum needed by Linux to initialise and think
//! interrupts are working, without actually generating any real interrupts
//! (except the timer IRQ, which is handled in the CPU execution loop).
//!
//! The GIC is split into two MMIO regions:
//!   - Distributor (GICD) at 0x0800_0000 — global interrupt configuration
//!   - Redistributor (GICR) at 0x080A_0000 — per-CPU interrupt handling
//!
//! The CPU interface (ICC_*) is accessed via system registers, not MMIO.

use crate::constants::*;

/// Number of 32-bit words for the enable/pending/group bitmap arrays.
const INT_WORDS: usize = 32;
/// Total number of individual interrupts supported (32 words × 32 bits).
const MAX_INTERRUPTS: usize = 1024;

pub struct Gicv3 {
    // ── Distributor (GICD) registers ──
    pub ctld: u64,               // GICD_CTLR  (0x0000)
    pub typer: u64,              // GICD_TYPER (0x0008, read-only)
    pub iidr: u32,               // GICD_IIDR  (0x0018, read-only)
    pub enable: [u32; INT_WORDS],   // ISENABLER / ICENABLER (0x0100–0x017C)
    pub pending: [u32; INT_WORDS],  // ISPENDR / ICPENDR     (0x0200–0x027C)
    pub priority: [u8; MAX_INTERRUPTS], // IPRIORITYR (0x0400–0x07FC)
    pub group: [u32; INT_WORDS],   // IGROUPR / IGRPMODR    (0x0800–0x087C)

    // ── Redistributor (GICR) registers ──
    pub rctlr: u64,              // GICR_CTLR
    pub rwaker: u64,             // GICR_WAKER
    pub rtyper: u64,             // GICR_TYPER (read-only)
}

impl Gicv3 {
    pub fn new() -> Self {
        Self {
            ctld: 0,
            typer: 0,                // ITLinesNumber = 0 → 32 interrupts
            iidr: GICD_IIDR_VAL,
            enable: [0; INT_WORDS],
            pending: [0; INT_WORDS],
            priority: [0; MAX_INTERRUPTS],
            group: [0; INT_WORDS],
            rctlr: 0,
            rwaker: 0,
            rtyper: 0,               // ProcessorNumber = 0
        }
    }

    /// Handle GICD (distributor) MMIO read.
    pub fn gicd_read(&self, offset: u64, size: u8) -> Option<u64> {
        match offset {
            0x0000 => Some(self.ctld),
            0x0004 => Some(self.ctld >> 32),
            0x0008 => Some(self.typer),
            0x0018 => Some(self.iidr as u64),

            o if gicd_in_range(o, 0x0100, 0x0180) => {
                let idx = gicd_word_index(o, 0x0100);
                Some(read_bitmap_word(&self.enable, idx))
            }
            o if gicd_in_range(o, 0x0200, 0x0280) => {
                let idx = gicd_word_index(o, 0x0200);
                Some(read_bitmap_word(&self.pending, idx))
            }
            o if gicd_in_range(o, 0x0400, 0x0800) => {
                let idx = (o - 0x0400) as usize;
                if idx < MAX_INTERRUPTS {
                    let mut val = 0u64;
                    for i in 0..4.min(size as usize) {
                        if idx + i < MAX_INTERRUPTS {
                            val |= (self.priority[idx + i] as u64) << (i * 8);
                        }
                    }
                    Some(val)
                } else {
                    Some(0)
                }
            }
            o if gicd_in_range(o, 0x0800, 0x0880) => {
                let idx = gicd_word_index(o, 0x0800);
                Some(read_bitmap_word(&self.group, idx))
            }
            _ => Some(0),
        }
    }

    /// Handle GICD (distributor) MMIO write.
    pub fn gicd_write(&mut self, offset: u64, value: u64, size: u8) {
        match offset {
            0x0000 => self.ctld = value,

            // ISENABLER (set-enable): write 1 to enable
            o if gicd_in_range(o, 0x0100, 0x0180) => {
                let idx = gicd_word_index(o, 0x0100);
                if idx < INT_WORDS { self.enable[idx] = value as u32; }
            }
            // ICENABLER (clear-enable): write 1 to disable
            o if gicd_in_range(o, 0x0180, 0x0200) => {
                let idx = gicd_word_index(o, 0x0180);
                if idx < INT_WORDS { self.enable[idx] &= !(value as u32); }
            }
            // ISPENDR (set-pending)
            o if gicd_in_range(o, 0x0200, 0x0280) => {
                let idx = gicd_word_index(o, 0x0200);
                if idx < INT_WORDS { self.pending[idx] = value as u32; }
            }
            // ICPENDR (clear-pending)
            o if gicd_in_range(o, 0x0280, 0x0300) => {
                let idx = gicd_word_index(o, 0x0280);
                if idx < INT_WORDS { self.pending[idx] &= !(value as u32); }
            }
            // IPRIORITYR (priority, 8-bit per interrupt)
            o if gicd_in_range(o, 0x0400, 0x0800) => {
                let idx = (o - 0x0400) as usize;
                for i in 0..(size as usize).min(4) {
                    if idx + i < MAX_INTERRUPTS {
                        self.priority[idx + i] = ((value >> (i * 8)) & 0xFF) as u8;
                    }
                }
            }
            // IGROUPR
            o if gicd_in_range(o, 0x0800, 0x0880) => {
                let idx = gicd_word_index(o, 0x0800);
                if idx < INT_WORDS { self.group[idx] = value as u32; }
            }
            _ => {}
        }
    }

    /// Handle GICR (redistributor) MMIO read.
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

    /// Handle GICR (redistributor) MMIO write.
    pub fn gicr_write(&mut self, offset: u64, value: u64, _size: u8) {
        match offset {
            0x0000 => self.rctlr = value,
            0x0014 => self.rwaker = value,
            _ => {}
        }
    }
}

// ── GICD register helpers ──

/// Returns true if `offset` falls within [base, end) as a register window.
fn gicd_in_range(offset: u64, base: u64, end: u64) -> bool {
    offset >= base && offset < end
}

/// Computes the 32-bit-word index within a GICD bitmap register array.
fn gicd_word_index(offset: u64, base: u64) -> usize {
    ((offset - base) / 4) as usize
}

/// Reads one word from a bitmap array, truncating to 0 if out of bounds.
fn read_bitmap_word(arr: &[u32; INT_WORDS], idx: usize) -> u64 {
    if idx < INT_WORDS { arr[idx] as u64 } else { 0 }
}
