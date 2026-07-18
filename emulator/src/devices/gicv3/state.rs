use super::GICR_TYPER_LAST;

pub(super) struct RedistributorState {
    pub ctlr: u64,
    pub waker: u64,
    pub enable: u32,
    pub pending: u32,
    pub active: u32,
    pub group: u32,
    pub priority: [u8; 32],
}

impl RedistributorState {
    pub fn new() -> Self {
        Self {
            ctlr: 0,
            waker: 0,
            enable: 0,
            pending: 0,
            active: 0,
            group: 0,
            priority: [0; 32],
        }
    }

    pub fn pending_enabled(&self) -> u32 {
        self.pending & self.enable & !self.active
    }
}

/// Compact Aff3:Aff2:Aff1:Aff0 value used by GICR_TYPER.
pub(super) fn affinity_value(cpu_id: usize) -> u32 {
    cpu_id as u32
}

/// MPIDR/IROUTER-shaped affinity (Aff3 occupies bits 39:32).
pub(super) fn route_affinity(cpu_id: usize) -> u64 {
    let affinity = affinity_value(cpu_id);
    let aff0_to_aff2 = (affinity & 0x00ff_ffff) as u64;
    let aff3 = ((affinity >> 24) as u64) << 32;
    aff3 | aff0_to_aff2
}

pub(super) fn redistributor_typer(cpu_id: usize, cpu_count: usize) -> u64 {
    let affinity = (affinity_value(cpu_id) as u64) << 32;
    let processor_number = ((cpu_id as u64) & 0xffff) << 8;
    let last = if cpu_id + 1 == cpu_count {
        GICR_TYPER_LAST
    } else {
        0
    };
    affinity | processor_number | last
}
