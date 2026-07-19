use super::*;
use std::sync::atomic::Ordering;

impl Machine {
    #[cfg(test)]
    pub(crate) fn parallel_wasm_active(&self) -> bool {
        self.wasm_parallel.active.load(Ordering::Acquire)
    }

    pub(crate) fn begin_parallel_wasm(
        &mut self,
        max_steps: usize,
        start: WasmParallelStart,
    ) -> Result<u64, &'static str> {
        let control = &self.wasm_parallel;
        control
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "parallel run already active")?;
        let generation = next_generation(control);
        control.start_budget.store(max_steps, Ordering::Relaxed);
        control.remaining.store(max_steps, Ordering::Relaxed);
        control
            .next_cycle
            .store(self.virtual_time, Ordering::Relaxed);
        control.stop.store(false, Ordering::Relaxed);
        control.reset_requested.store(false, Ordering::Relaxed);
        control.system_off.store(false, Ordering::Relaxed);
        control
            .memory_epoch
            .store(self.memory_epoch, Ordering::Relaxed);
        control.workers_arrived.store(0, Ordering::Relaxed);
        control.workers_completed.store(0, Ordering::Relaxed);
        control.workers_in_flight.store(0, Ordering::Relaxed);
        control.fetch_faults.store(0, Ordering::Relaxed);
        control.exec_faults.store(0, Ordering::Relaxed);
        control.local_in_flight.store(0, Ordering::Relaxed);
        control.max_local_in_flight.store(0, Ordering::Relaxed);
        control
            .cpu_base
            .store(self.cpus.as_mut_ptr() as usize as u64, Ordering::Relaxed);
        control
            .cache_base
            .store(self.caches.as_mut_ptr() as usize as u64, Ordering::Relaxed);
        control.bus_ptr.store(
            (&mut self.bus as *mut SystemBus) as usize as u64,
            Ordering::Relaxed,
        );
        for (core, cpu) in self.cpus.iter().enumerate() {
            let lifecycle = lifecycle_code(cpu);
            control.core_owners[core].store(0, Ordering::Relaxed);
            control.event_registers[core].store(cpu.event_register, Ordering::Relaxed);
            control.deadlines[core]
                .store(idle::initial_deadline(cpu, lifecycle), Ordering::Relaxed);
            control.lifecycle[core].store(lifecycle, Ordering::Relaxed);
        }
        let token = registry::register(self, generation, start.control());
        self.wasm_parallel.run_token.store(token, Ordering::Release);
        start.commit();
        Ok(token)
    }

    pub(crate) fn run_parallel_wasm_core(token: u64, core: usize) -> Result<(), &'static str> {
        let lease = registry::claim(token, core)?;
        worker::run(&lease)
    }

    pub(crate) fn cancel_parallel_wasm(token: u64) -> Result<(), &'static str> {
        registry::cancel(token)
    }

    pub(crate) fn finish_parallel_wasm(token: u64) -> Result<(usize, u64), &'static str> {
        let claim = registry::close(token)?;
        let machine_ptr = claim.machine();
        let machine = unsafe { &mut *machine_ptr };
        let steps = machine.finalize_parallel_wasm();
        let result = (steps, machine.cpus[0].regs.pc);
        registry::complete_finalize(claim);
        Ok(result)
    }

    fn finalize_parallel_wasm(&mut self) -> usize {
        let control = &self.wasm_parallel;
        let budget = control.start_budget.load(Ordering::Acquire);
        let ran = budget - control.remaining.load(Ordering::Acquire);
        self.memory_epoch = control.memory_epoch.load(Ordering::Acquire);
        self.virtual_time = control.next_cycle.load(Ordering::Acquire);
        self.fetch_faults += control.fetch_faults.load(Ordering::Relaxed);
        self.exec_faults += control.exec_faults.load(Ordering::Relaxed);
        self.total_steps = self.total_steps.saturating_add(ran as u64);
        self.parallel_stats = ParallelRunStats {
            worker_threads: control.workers_arrived.load(Ordering::Relaxed),
            max_local_in_flight: control.max_local_in_flight.load(Ordering::Relaxed),
        };
        let reset_requested = control.reset_requested.load(Ordering::Acquire)
            && !control.system_off.load(Ordering::Acquire);
        sync_lifecycle(self);
        if reset_requested {
            self.psci_system_reset();
        }
        ran
    }
}

fn next_generation(control: &WasmParallelControl) -> u64 {
    let previous = control
        .generation
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
            Some(if value == u64::MAX { 1 } else { value + 1 })
        })
        .expect("generation update cannot fail");
    if previous == u64::MAX {
        1
    } else {
        previous + 1
    }
}

fn sync_lifecycle(machine: &mut Machine) {
    if machine.wasm_parallel.system_off.load(Ordering::Acquire) {
        for cpu in &mut machine.cpus {
            cpu.lifecycle = CpuLifecycle::PoweredOff;
            cpu.event_register = false;
            cpu.waiting_for_event = false;
        }
        return;
    }
    for core in 0..machine.cpus.len() {
        machine.cpus[core].event_register =
            machine.wasm_parallel.event_registers[core].load(Ordering::Acquire);
        machine.cpus[core].waiting_for_event = false;
        match machine.wasm_parallel.lifecycle[core].load(Ordering::Acquire) {
            LIFE_OFF => machine.cpus[core].lifecycle = CpuLifecycle::PoweredOff,
            LIFE_WAITING => machine.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt,
            LIFE_WAITING_EVENT => {
                machine.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
                machine.cpus[core].waiting_for_event = true;
            }
            LIFE_STARTING | LIFE_BOOT_READY => initialize_powered_on_core(machine, core),
            _ => machine.cpus[core].lifecycle = CpuLifecycle::Runnable,
        }
    }
}

fn initialize_powered_on_core(machine: &mut Machine, core: usize) {
    let entry = machine.wasm_parallel.power_entry[core].load(Ordering::Acquire);
    let context = machine.wasm_parallel.power_context[core].load(Ordering::Acquire);
    let cpu = &mut machine.cpus[core];
    cpu.reset();
    machine.caches[core] = DecodeCache::new();
    cpu.lifecycle = CpuLifecycle::Runnable;
    cpu.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    cpu.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
    cpu.sys.cycle_count = machine.virtual_time;
    cpu.regs.pc = entry;
    cpu.regs.set_x(0, context);
}

fn lifecycle_code(cpu: &Armv8Cpu) -> u8 {
    match cpu.lifecycle {
        CpuLifecycle::PoweredOff => LIFE_OFF,
        CpuLifecycle::Runnable => LIFE_RUNNABLE,
        CpuLifecycle::WaitingForInterrupt if cpu.waiting_for_event => LIFE_WAITING_EVENT,
        CpuLifecycle::WaitingForInterrupt => LIFE_WAITING,
    }
}
