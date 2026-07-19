use super::*;

impl Machine {
    pub(crate) fn prepare_next_core(&mut self) -> Option<usize> {
        self.apply_external_dma_write_invalidations();
        self.wake_waiting_cores();
        if self.find_runnable_from(self.active_core).is_none() {
            self.advance_idle_time();
            self.wake_waiting_cores();
        }
        let core = self.find_runnable_from(self.active_core)?;
        self.active_core = core;
        self.cpus[core].sys.cycle_count = self.virtual_time;
        Some(core)
    }

    pub(super) fn find_runnable_from(&self, start: usize) -> Option<usize> {
        let count = self.cpus.len();
        (0..count)
            .map(|offset| (start + offset) % count)
            .find(|&core| self.cpus[core].lifecycle == CpuLifecycle::Runnable)
    }

    pub(super) fn wake_waiting_cores(&mut self) {
        self.bus.refresh_interrupts();
        for core in 0..self.cpus.len() {
            if self.cpus[core].lifecycle != CpuLifecycle::WaitingForInterrupt {
                continue;
            }
            self.cpus[core].sys.cycle_count = self.virtual_time;
            if self.core_has_wake_event(core) {
                self.cpus[core].lifecycle = CpuLifecycle::Runnable;
            }
        }
    }

    pub(super) fn park_after_wfi(&mut self, core: usize) {
        if !self.core_has_wake_event(core) {
            self.cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
        }
    }

    fn advance_idle_time(&mut self) {
        let deadline = self
            .cpus
            .iter()
            .filter(|cpu| cpu.lifecycle == CpuLifecycle::WaitingForInterrupt)
            .filter_map(|cpu| cpu.sys.next_timer_deadline())
            .min();
        if let Some(deadline) = deadline {
            self.virtual_time = self.virtual_time.max(deadline);
        }
    }

    fn core_has_wake_event(&self, core: usize) -> bool {
        let cpu = &self.cpus[core];
        cpu.sys.irq_pending
            || cpu.sys.timer_irq_check_needed()
            || self.bus.gic.has_pending_enabled_for_cpu(core)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runnable_scan_skips_off_and_waiting_cores() {
        let mut machine = Machine::new(3);
        machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
        machine.cpus[2].lifecycle = CpuLifecycle::Runnable;

        assert_eq!(machine.find_runnable_from(0), Some(2));
        assert_eq!(machine.find_runnable_from(1), Some(2));
    }

    #[test]
    fn no_runnable_core_is_a_bounded_idle_state() {
        let mut machine = Machine::new(2);
        machine.cpus[0].lifecycle = CpuLifecycle::PoweredOff;

        assert_eq!(machine.prepare_next_core(), None);
    }

    #[test]
    fn idle_time_fast_forwards_only_when_every_cpu_is_asleep() {
        let mut machine = Machine::new(2);
        machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
        machine.cpus[0].sys.vbar_el1 = 0x8000;
        machine.cpus[0].sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
        machine.cpus[0].sys.cntv_cval_el0 = 100;
        machine.cpus[1].lifecycle = CpuLifecycle::Runnable;

        assert_eq!(machine.prepare_next_core(), Some(1));
        assert_eq!(machine.virtual_time, 0);

        machine.cpus[1].lifecycle = CpuLifecycle::WaitingForInterrupt;
        assert_eq!(machine.prepare_next_core(), Some(0));
        assert_eq!(machine.virtual_time, 100);
    }
}
