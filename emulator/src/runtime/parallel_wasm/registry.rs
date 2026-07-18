use super::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

struct RunEntry {
    token: u64,
    generation: u64,
    machine: usize,
    access: Arc<WasmAccessControl>,
}

static RUNS: Mutex<Vec<RunEntry>> = Mutex::new(Vec::new());
static NEXT_TOKEN: AtomicU64 = AtomicU64::new(0x6d75_6c74_6963_6f72);

pub(super) struct WorkerLease {
    control: *const WasmParallelControl,
    pub(super) core: usize,
}

pub(super) struct FinalizeClaim {
    machine: *mut Machine,
    token: u64,
    access: Arc<WasmAccessControl>,
}

impl WorkerLease {
    pub(super) fn control(&self) -> &WasmParallelControl {
        // The registry retains this run until every lease has been dropped.
        unsafe { &*self.control }
    }
}

impl Drop for WorkerLease {
    fn drop(&mut self) {
        let control = unsafe { &*self.control };
        control.workers_completed.fetch_add(1, Ordering::Release);
        control.workers_in_flight.fetch_sub(1, Ordering::Release);
    }
}

pub(super) fn register(
    machine: &mut Machine,
    generation: u64,
    access: Arc<WasmAccessControl>,
) -> u64 {
    let mut runs = lock_runs();
    let machine_address = machine as *mut Machine as usize;
    debug_assert!(!runs.iter().any(|entry| entry.machine == machine_address));
    let token = next_unused_token(&runs);
    runs.push(RunEntry {
        token,
        generation,
        machine: machine_address,
        access,
    });
    token
}

pub(super) fn claim(token: u64, core: usize) -> Result<WorkerLease, &'static str> {
    let runs = lock_runs();
    let entry = runs
        .iter()
        .find(|entry| entry.token == token)
        .ok_or("invalid or expired parallel run token")?;
    entry.access.require_parallel_run()?;
    let control = unsafe { control_at(entry.machine) };
    validate_entry(entry, control)?;
    let owner = control
        .core_owners
        .get(core)
        .ok_or("parallel vCPU index is out of range")?;
    owner
        .compare_exchange(0, entry.generation, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| "parallel vCPU already claimed for this run")?;
    control.workers_in_flight.fetch_add(1, Ordering::AcqRel);
    control.workers_arrived.fetch_add(1, Ordering::AcqRel);
    Ok(WorkerLease { control, core })
}

pub(super) fn cancel(token: u64) -> Result<(), &'static str> {
    let runs = lock_runs();
    let entry = runs
        .iter()
        .find(|entry| entry.token == token)
        .ok_or("invalid or expired parallel run token")?;
    entry.access.require_parallel_run()?;
    let control = unsafe { control_at(entry.machine) };
    validate_entry(entry, control)?;
    control.stop.store(true, Ordering::Release);
    Ok(())
}

pub(super) fn close(token: u64) -> Result<FinalizeClaim, &'static str> {
    let mut runs = lock_runs();
    let index = runs
        .iter()
        .position(|entry| entry.token == token)
        .ok_or("invalid or expired parallel run token")?;
    let entry = &runs[index];
    entry.access.require_parallel_run()?;
    let machine = entry.machine as *mut Machine;
    let control = unsafe { control_at(entry.machine) };
    validate_entry(entry, control)?;
    let started = control.workers_arrived.load(Ordering::Acquire);
    let completed = control.workers_completed.load(Ordering::Acquire);
    if control.workers_in_flight.load(Ordering::Acquire) != 0 || completed != started {
        return Err("parallel workers have not quiesced");
    }
    if started < control.lifecycle.len() && !control.stop.load(Ordering::Acquire) {
        return Err("parallel workers have not all started");
    }
    let entry = runs.swap_remove(index);
    Ok(FinalizeClaim {
        machine,
        token,
        access: entry.access,
    })
}

pub(super) fn complete_finalize(claim: FinalizeClaim) {
    let control = unsafe { &*std::ptr::addr_of!((*claim.machine).wasm_parallel) };
    debug_assert!(control.active.load(Ordering::Acquire));
    debug_assert_eq!(control.run_token.load(Ordering::Acquire), claim.token);
    control.run_token.store(0, Ordering::Release);
    control.active.store(false, Ordering::Release);
    // Publish external idleness only after the final Machine-field access.
    claim.access.finish_parallel();
}

impl FinalizeClaim {
    pub(super) fn machine(&self) -> *mut Machine {
        self.machine
    }
}

fn validate_entry(entry: &RunEntry, control: &WasmParallelControl) -> Result<(), &'static str> {
    if !control.active.load(Ordering::Acquire)
        || control.run_token.load(Ordering::Acquire) != entry.token
        || control.generation.load(Ordering::Acquire) != entry.generation
    {
        return Err("stale parallel run token");
    }
    Ok(())
}

fn lock_runs() -> std::sync::MutexGuard<'static, Vec<RunEntry>> {
    RUNS.lock().unwrap_or_else(|poison| poison.into_inner())
}

unsafe fn control_at(machine: usize) -> &'static WasmParallelControl {
    let machine = machine as *const Machine;
    unsafe { &*std::ptr::addr_of!((*machine).wasm_parallel) }
}

fn next_unused_token(runs: &[RunEntry]) -> u64 {
    loop {
        let token = NEXT_TOKEN
            .fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed)
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .max(1);
        if runs.iter().all(|entry| entry.token != token) {
            return token;
        }
    }
}
