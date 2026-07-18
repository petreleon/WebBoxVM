use super::{IrqId, VirtAddr};
use crate::runtime::Machine;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
use crate::runtime::RunBackend;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmConfig {
    cores: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmConfigError {
    ZeroCores,
    TooManyCores { requested: usize, maximum: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmMetrics {
    pub total_steps: u64,
    pub pc: VirtAddr,
    pub allocated_pages: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmEvent {
    UartOutput(Vec<u8>),
    Metrics(VmMetrics),
}

pub struct VmHandle {
    config: VmConfig,
    machine: Machine,
}

impl VmConfig {
    pub fn new(cores: usize) -> Result<Self, VmConfigError> {
        if cores == 0 {
            Err(VmConfigError::ZeroCores)
        } else if cores > crate::constants::GICR_MAX_CPUS {
            Err(VmConfigError::TooManyCores {
                requested: cores,
                maximum: crate::constants::GICR_MAX_CPUS,
            })
        } else {
            Ok(Self { cores })
        }
    }

    pub const fn single_core() -> Self {
        Self { cores: 1 }
    }

    pub const fn cores(self) -> usize {
        self.cores
    }
}

impl Default for VmConfig {
    fn default() -> Self {
        Self::single_core()
    }
}

impl VmHandle {
    pub fn new(config: VmConfig) -> Self {
        let machine = Machine::new(config.cores());
        #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
        let machine = {
            let mut machine = machine;
            if config.cores() > 1 {
                machine.set_run_backend(RunBackend::NativeThreads);
            }
            machine
        };
        Self { machine, config }
    }

    pub fn try_new(cores: usize) -> Result<Self, VmConfigError> {
        VmConfig::new(cores).map(Self::new)
    }

    pub const fn config(&self) -> VmConfig {
        self.config
    }

    pub fn metrics(&self) -> VmMetrics {
        VmMetrics {
            total_steps: self.machine.total_steps,
            pc: VirtAddr::new(self.machine.cpus[0].regs.pc),
            allocated_pages: self.machine.bus.mem.allocated_pages(),
        }
    }

    pub fn run_steps(&mut self, max_steps: usize) -> usize {
        self.machine.run(max_steps)
    }

    pub fn inject_irq(&mut self, irq: IrqId) {
        self.machine.inject_irq(irq.get());
    }
}

impl Default for VmHandle {
    fn default() -> Self {
        Self::new(VmConfig::default())
    }
}

impl fmt::Debug for VmHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmHandle")
            .field("config", &self.config)
            .field("metrics", &self.metrics())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_config_rejects_zero_cores() {
        assert_eq!(VmConfig::new(0), Err(VmConfigError::ZeroCores));
        assert_eq!(VmConfig::new(2).unwrap().cores(), 2);
    }

    #[test]
    fn vm_config_rejects_counts_beyond_the_gicr_aperture() {
        let requested = crate::constants::GICR_MAX_CPUS + 1;

        assert_eq!(
            VmConfig::new(requested),
            Err(VmConfigError::TooManyCores {
                requested,
                maximum: crate::constants::GICR_MAX_CPUS,
            })
        );
    }

    #[test]
    fn vm_handle_constructs_runtime_machine() {
        let handle = VmHandle::new(VmConfig::new(2).unwrap());

        assert_eq!(handle.config().cores(), 2);
        assert_eq!(handle.machine.cpus.len(), 2);
        assert_eq!(handle.metrics().pc, VirtAddr::new(0));
    }

    #[test]
    fn vm_handle_reports_runtime_metrics() {
        let mut handle = VmHandle::default();
        handle.machine.total_steps = 7;
        handle.machine.cpus[0].regs.pc = 0x4008_0000;
        handle.machine.bus.mem.write(0x4000_0000, 1, 0xaa);

        assert_eq!(
            handle.metrics(),
            VmMetrics {
                total_steps: 7,
                pc: VirtAddr::new(0x4008_0000),
                allocated_pages: 1,
            }
        );
    }

    #[test]
    fn vm_events_carry_typed_metrics() {
        let metrics = VmMetrics {
            total_steps: 9,
            pc: VirtAddr::new(0x4000_0000),
            allocated_pages: 1,
        };
        assert_eq!(VmEvent::Metrics(metrics), VmEvent::Metrics(metrics));
    }
}
