//! Shared-Memory64 vCPU executor used by browser Web Workers.

use super::*;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

mod idle;
mod instruction;
mod machine;
mod psci;
mod registry;
mod worker;

#[cfg(test)]
mod tests;

pub(super) const LIFE_OFF: u8 = 0;
pub(super) const LIFE_RUNNABLE: u8 = 1;
pub(super) const LIFE_WAITING: u8 = 2;
pub(super) const LIFE_STARTING: u8 = 3;
pub(super) const LIFE_BOOT_READY: u8 = 4;
pub(super) const NO_DEADLINE: u64 = u64::MAX;

pub(crate) struct WasmParallelControl {
    pub gate: RwLock<()>,
    pub active: AtomicBool,
    pub run_token: AtomicU64,
    pub generation: AtomicU64,
    pub start_budget: AtomicUsize,
    pub remaining: AtomicUsize,
    pub next_cycle: AtomicU64,
    pub stop: AtomicBool,
    pub reset_requested: AtomicBool,
    pub system_off: AtomicBool,
    pub memory_epoch: AtomicU64,
    pub tlb_epoch: AtomicU64,
    pub lifecycle: Vec<AtomicU8>,
    pub deadlines: Vec<AtomicU64>,
    pub power_entry: Vec<AtomicU64>,
    pub power_context: Vec<AtomicU64>,
    pub cpu_base: AtomicU64,
    pub cache_base: AtomicU64,
    pub bus_ptr: AtomicU64,
    pub workers_arrived: AtomicUsize,
    pub workers_completed: AtomicUsize,
    pub workers_in_flight: AtomicUsize,
    pub core_owners: Vec<AtomicU64>,
    pub fetch_faults: AtomicU64,
    pub exec_faults: AtomicU64,
    pub local_in_flight: AtomicUsize,
    pub max_local_in_flight: AtomicUsize,
}

impl WasmParallelControl {
    pub(crate) fn new(cores: usize) -> Self {
        Self {
            gate: RwLock::new(()),
            active: AtomicBool::new(false),
            run_token: AtomicU64::new(0),
            generation: AtomicU64::new(0),
            start_budget: AtomicUsize::new(0),
            remaining: AtomicUsize::new(0),
            next_cycle: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            reset_requested: AtomicBool::new(false),
            system_off: AtomicBool::new(false),
            memory_epoch: AtomicU64::new(0),
            tlb_epoch: AtomicU64::new(0),
            lifecycle: atomics_u8(cores, LIFE_OFF),
            deadlines: atomics_u64(cores, NO_DEADLINE),
            power_entry: atomics_u64(cores, 0),
            power_context: atomics_u64(cores, 0),
            cpu_base: AtomicU64::new(0),
            cache_base: AtomicU64::new(0),
            bus_ptr: AtomicU64::new(0),
            workers_arrived: AtomicUsize::new(0),
            workers_completed: AtomicUsize::new(0),
            workers_in_flight: AtomicUsize::new(0),
            core_owners: atomics_u64(cores, 0),
            fetch_faults: AtomicU64::new(0),
            exec_faults: AtomicU64::new(0),
            local_in_flight: AtomicUsize::new(0),
            max_local_in_flight: AtomicUsize::new(0),
        }
    }

    pub fn claim_step(&self) -> bool {
        self.remaining
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |left| {
                left.checked_sub(1)
            })
            .is_ok()
    }

    pub fn observe_local_overlap(&self, entering: bool) {
        if !entering {
            self.local_in_flight.fetch_sub(1, Ordering::Release);
            return;
        }
        let active = self.local_in_flight.fetch_add(1, Ordering::AcqRel) + 1;
        self.max_local_in_flight
            .fetch_max(active, Ordering::Relaxed);
    }
}

fn atomics_u64(len: usize, value: u64) -> Vec<AtomicU64> {
    (0..len).map(|_| AtomicU64::new(value)).collect()
}

fn atomics_u8(len: usize, value: u8) -> Vec<AtomicU8> {
    (0..len).map(|_| AtomicU8::new(value)).collect()
}
