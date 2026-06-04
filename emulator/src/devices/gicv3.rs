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

mod distributor;
mod redistributor;

/// Number of 32-bit words for the enable/pending/group bitmap arrays.
const INT_WORDS: usize = 32;
/// Total number of individual interrupts supported (32 words × 32 bits).
const MAX_INTERRUPTS: usize = 1024;
/// GICD/GICR PIDR2 high nibble identifying a GICv3-compatible controller.
const GIC_PIDR2_ARCH_GICV3: u64 = 0x30;
/// GICR_TYPER.Last: this single redistributor frame is the last one.
const GICR_TYPER_LAST: u64 = 1 << 4;

pub struct Gicv3 {
    // ── Distributor (GICD) registers ──
    pub ctld: u64,                      // GICD_CTLR  (0x0000)
    pub typer: u64,                     // GICD_TYPER (0x0008, read-only)
    pub iidr: u32,                      // GICD_IIDR  (0x0018, read-only)
    pub enable: [u32; INT_WORDS],       // ISENABLER / ICENABLER (0x0100–0x017C)
    pub pending: [u32; INT_WORDS],      // ISPENDR / ICPENDR     (0x0200–0x027C)
    pub priority: [u8; MAX_INTERRUPTS], // IPRIORITYR (0x0400–0x07FC)
    pub group: [u32; INT_WORDS],        // IGROUPR / IGRPMODR    (0x0800–0x087C)

    // ── Redistributor (GICR) registers ──
    pub rctlr: u64,  // GICR_CTLR
    pub rwaker: u64, // GICR_WAKER
    pub rtyper: u64, // GICR_TYPER (read-only)
}

impl Gicv3 {
    pub fn new() -> Self {
        Self {
            ctld: 0,
            typer: 1, // ITLinesNumber = 1 → 64 interrupts
            iidr: GICD_IIDR_VAL,
            enable: [0; INT_WORDS],
            pending: [0; INT_WORDS],
            priority: [0; MAX_INTERRUPTS],
            group: [0; INT_WORDS],
            rctlr: 0,
            rwaker: 0,
            rtyper: GICR_TYPER_LAST, // ProcessorNumber = 0, last redistributor
        }
    }

    pub fn set_pending(&mut self, int_id: u32) {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        if idx < INT_WORDS {
            self.pending[idx] |= bit;
        }
    }

    pub fn clear_pending(&mut self, int_id: u32) {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        if idx < INT_WORDS {
            self.pending[idx] &= !bit;
        }
    }

    pub fn next_pending_enabled(&self) -> Option<u32> {
        for idx in 0..INT_WORDS {
            let active = self.pending[idx] & self.enable[idx];
            if active != 0 {
                return Some((idx as u32) * 32 + active.trailing_zeros());
            }
        }
        None
    }
}

pub(in crate::devices::gicv3) fn gicd_in_range(offset: u64, base: u64, end: u64) -> bool {
    offset >= base && offset < end
}

pub(in crate::devices::gicv3) fn gicd_word_index(offset: u64, base: u64) -> usize {
    ((offset - base) / 4) as usize
}

pub(in crate::devices::gicv3) fn read_bitmap_word(arr: &[u32; INT_WORDS], idx: usize) -> u64 {
    if idx < INT_WORDS { arr[idx] as u64 } else { 0 }
}
