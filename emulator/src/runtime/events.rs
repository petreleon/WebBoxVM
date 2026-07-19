use super::*;

impl Machine {
    /// Apply the architected event-register effect after a retired hint.
    pub(super) fn finish_event_instruction(&mut self, core: usize, op: Opcode) {
        match op {
            Opcode::Sevl => self.cpus[core].event_register = true,
            Opcode::Sev => self.broadcast_event(),
            Opcode::Wfe => self.park_after_wfe(core),
            Opcode::Wfi => self.park_after_wfi(core),
            _ => {}
        }
    }

    fn broadcast_event(&mut self) {
        for core in 0..self.cpus.len() {
            self.signal_event(core);
        }
    }

    pub(super) fn signal_event(&mut self, core: usize) {
        signal_cpu_event(&mut self.cpus[core]);
    }

    fn park_after_wfe(&mut self, core: usize) {
        if self.cpus[core].event_register {
            self.cpus[core].event_register = false;
            self.cpus[core].waiting_for_event = false;
            return;
        }
        if self.core_has_wake_event(core) {
            self.cpus[core].waiting_for_event = false;
            return;
        }
        self.cpus[core].waiting_for_event = true;
        self.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
        self.cooperative_wfe_parks = self.cooperative_wfe_parks.saturating_add(1);
    }

    fn park_after_wfi(&mut self, core: usize) {
        self.cpus[core].waiting_for_event = false;
        if !self.core_has_wake_event(core) {
            self.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
        }
    }
}

pub(super) fn signal_cpu_event(cpu: &mut Armv8Cpu) {
    let wakes_wfe = cpu.lifecycle == CpuLifecycle::WaitingForInterrupt && cpu.waiting_for_event;
    cpu.event_register = !wakes_wfe;
    if wakes_wfe {
        cpu.lifecycle = CpuLifecycle::Runnable;
        cpu.waiting_for_event = false;
    }
}
