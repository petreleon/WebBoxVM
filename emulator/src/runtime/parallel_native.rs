//! Native host-parallel vCPU executor.
//!
//! Each worker exclusively owns one CPU and decode cache. Shared RAM/MMIO is
//! accessed through a short-lived bus lock, while proven CPU-local operations
//! execute concurrently.

use super::*;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

pub(super) mod idle;
mod instruction;
mod lifecycle;
pub(super) mod psci;
pub(in crate::runtime) mod spawn;
mod worker;

pub(super) use lifecycle::{initialize_powered_on_core, lifecycle_code};

pub(super) const LIFE_OFF: u8 = 0;
pub(super) const LIFE_RUNNABLE: u8 = 1;
pub(super) const LIFE_WAITING: u8 = 2;
pub(super) const LIFE_STARTING: u8 = 3;
pub(super) const LIFE_BOOT_READY: u8 = 4;
pub(super) const NO_DEADLINE: u64 = u64::MAX;

pub(super) struct SharedRun<'a> {
    pub bus: RwLock<&'a mut SystemBus>,
    pub idle_gate: RwLock<()>,
    pub remaining: AtomicUsize,
    pub next_cycle: AtomicU64,
    pub stop: AtomicBool,
    pub system_off: AtomicBool,
    pub reset_requested: AtomicBool,
    pub memory_epoch: AtomicU64,
    pub tlb_epoch: AtomicU64,
    pub lifecycle: Vec<AtomicU8>,
    pub deadlines: Vec<AtomicU64>,
    pub power_entry: Vec<AtomicU64>,
    pub power_context: Vec<AtomicU64>,
    pub fetch_faults: AtomicU64,
    pub exec_faults: AtomicU64,
    pub local_in_flight: AtomicUsize,
    pub max_local_in_flight: AtomicUsize,
}

impl SharedRun<'_> {
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

impl Machine {
    pub(super) fn run_parallel_native(&mut self, max_steps: usize) -> usize {
        if max_steps == 0 {
            return 0;
        }
        let core_count = self.cpus.len();
        let initial_state = self
            .cpus
            .iter()
            .map(|cpu| (lifecycle_code(cpu.lifecycle), idle::initial_deadline(cpu)))
            .collect::<Vec<_>>();
        let shared = SharedRun {
            bus: RwLock::new(&mut self.bus),
            idle_gate: RwLock::new(()),
            remaining: AtomicUsize::new(max_steps),
            next_cycle: AtomicU64::new(self.virtual_time),
            stop: AtomicBool::new(false),
            system_off: AtomicBool::new(false),
            reset_requested: AtomicBool::new(false),
            memory_epoch: AtomicU64::new(self.memory_epoch),
            tlb_epoch: AtomicU64::new(0),
            lifecycle: initial_state
                .iter()
                .map(|(state, _)| AtomicU8::new(*state))
                .collect(),
            deadlines: initial_state
                .iter()
                .map(|(_, deadline)| AtomicU64::new(*deadline))
                .collect(),
            power_entry: (0..core_count).map(|_| AtomicU64::new(0)).collect(),
            power_context: (0..core_count).map(|_| AtomicU64::new(0)).collect(),
            fetch_faults: AtomicU64::new(0),
            exec_faults: AtomicU64::new(0),
            local_in_flight: AtomicUsize::new(0),
            max_local_in_flight: AtomicUsize::new(0),
        };
        let (worker_threads, spawn_failed) = std::thread::scope(|scope| {
            let mut worker_threads = 0;
            let mut spawn_failed = false;
            for (core, (cpu, cache)) in self.cpus.iter_mut().zip(self.caches.iter_mut()).enumerate()
            {
                if spawn::should_fail(core) {
                    spawn_failed = true;
                    shared.stop.store(true, Ordering::Release);
                    break;
                }
                let shared = &shared;
                let spawned = std::thread::Builder::new()
                    .name(format!("webbox-vcpu-{core}"))
                    .spawn_scoped(scope, move || worker::run(core, cpu, cache, shared));
                match spawned {
                    Ok(_) => worker_threads += 1,
                    Err(_) => {
                        spawn_failed = true;
                        shared.stop.store(true, Ordering::Release);
                        break;
                    }
                }
            }
            (worker_threads, spawn_failed)
        });

        let remaining = shared.remaining.load(Ordering::Acquire);
        let ran = max_steps - remaining;
        let system_off = shared.system_off.load(Ordering::Acquire);
        let reset = !system_off && shared.reset_requested.load(Ordering::Acquire);
        self.memory_epoch = shared.memory_epoch.load(Ordering::Acquire);
        self.virtual_time = shared.next_cycle.load(Ordering::Acquire);
        self.fetch_faults += shared.fetch_faults.load(Ordering::Relaxed);
        self.exec_faults += shared.exec_faults.load(Ordering::Relaxed);
        self.parallel_stats = ParallelRunStats {
            worker_threads,
            max_local_in_flight: shared.max_local_in_flight.load(Ordering::Relaxed),
        };
        for core in 0..core_count {
            if system_off {
                self.cpus[core].lifecycle = CpuLifecycle::PoweredOff;
                continue;
            }
            match shared.lifecycle[core].load(Ordering::Acquire) {
                LIFE_OFF => self.cpus[core].lifecycle = CpuLifecycle::PoweredOff,
                LIFE_WAITING => {
                    self.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
                }
                LIFE_STARTING | LIFE_BOOT_READY => {
                    let entry = shared.power_entry[core].load(Ordering::Acquire);
                    let context = shared.power_context[core].load(Ordering::Acquire);
                    initialize_powered_on_core(
                        &mut self.cpus[core],
                        &mut self.caches[core],
                        entry,
                        context,
                        self.virtual_time,
                    );
                }
                _ => self.cpus[core].lifecycle = CpuLifecycle::Runnable,
            }
        }
        drop(shared);
        self.total_steps = self.total_steps.saturating_add(ran as u64);
        if spawn_failed {
            self.run_backend = RunBackend::Serial;
        }
        if reset {
            self.psci_system_reset();
        }
        if spawn_failed && !system_off && !reset {
            return ran + self.run_serial(remaining);
        }
        ran
    }
}
