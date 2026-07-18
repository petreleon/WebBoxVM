//! GICv3 interrupt controller.
//!
//! The controller is split into two MMIO regions:
//!   - Distributor (GICD) at 0x0800_0000 — global interrupt configuration
//!   - Redistributors (GICR) at 0x080A_0000 — one 128 KiB frame per CPU
//! The system-register CPU interface is implemented by the ARM64 core.

use crate::constants::*;

mod distributor;
mod lifecycle;
mod pending;
mod redistributor;
mod routing;
mod state;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_multicore;
#[cfg(test)]
mod tests_routing;

use state::{RedistributorState, redistributor_typer};

/// Number of 32-bit words for the enable/pending/group bitmap arrays.
const INT_WORDS: usize = 32;
/// Total number of individual interrupts supported (32 words × 32 bits).
const MAX_INTERRUPTS: usize = 1024;
/// SGIs and PPIs are private to a redistributor.
const PRIVATE_INTERRUPTS: usize = 32;
/// First shared peripheral interrupt.
const SPI_FIRST: u32 = 32;
/// GICD/GICR PIDR2 high nibble identifying a GICv3-compatible controller.
const GIC_PIDR2_ARCH_GICV3: u64 = 0x30;
/// GICR_TYPER.Last.
const GICR_TYPER_LAST: u64 = 1 << 4;

pub struct Gicv3 {
    // ── Distributor (GICD) registers ──
    pub ctld: u64,             // GICD_CTLR  (0x0000)
    pub typer: u64,            // GICD_TYPER (0x0008, read-only)
    pub iidr: u32,             // GICD_IIDR  (0x0018, read-only)
    enable: [u32; INT_WORDS],  // ISENABLER / ICENABLER (0x0100–0x017C)
    pending: [u32; INT_WORDS], // ISPENDR / ICPENDR     (0x0200–0x027C)
    active: [u32; INT_WORDS],
    pending_enabled: [u32; INT_WORDS],
    pending_enabled_words: u32,
    pub priority: [u8; MAX_INTERRUPTS], // IPRIORITYR (0x0400–0x07FC)
    pub group: [u32; INT_WORDS],        // IGROUPR (0x0080–0x00FC)
    irouter: [u64; MAX_INTERRUPTS],

    // CPU0 mirrors remain public for compatibility with the original model.
    pub rctlr: u64,
    pub rwaker: u64,
    pub rtyper: u64,
    redistributors: Vec<RedistributorState>,
}

impl Gicv3 {
    /// Construct the legacy single-CPU controller.
    pub fn new() -> Self {
        Self::with_cpu_count(1)
    }

    /// Construct a controller with one redistributor per CPU.
    ///
    /// A zero count is normalised to one: a GIC with no participating CPU
    /// cannot service an interrupt and is not a useful machine configuration.
    pub fn with_cpu_count(cpu_count: usize) -> Self {
        let cpu_count = cpu_count.max(1);
        assert!(
            cpu_count <= GICR_MAX_CPUS,
            "GIC CPU count exceeds the redistributor MMIO aperture"
        );
        Self {
            ctld: 0,
            typer: 1, // ITLinesNumber = 1 → 64 interrupts
            iidr: GICD_IIDR_VAL,
            enable: [0; INT_WORDS],
            pending: [0; INT_WORDS],
            active: [0; INT_WORDS],
            pending_enabled: [0; INT_WORDS],
            pending_enabled_words: 0,
            priority: [0; MAX_INTERRUPTS],
            group: [0; INT_WORDS],
            irouter: [0; MAX_INTERRUPTS], // Affinity 0 routes SPIs to CPU0.
            rctlr: 0,
            rwaker: 0,
            rtyper: redistributor_typer(0, cpu_count),
            redistributors: (0..cpu_count).map(|_| RedistributorState::new()).collect(),
        }
    }

    pub fn cpu_count(&self) -> usize {
        self.redistributors.len()
    }
}

impl Default for Gicv3 {
    fn default() -> Self {
        Self::new()
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
