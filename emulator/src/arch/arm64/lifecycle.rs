/// Scheduler-visible power and wait state for one emulated CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuLifecycle {
    PoweredOff,
    #[default]
    Runnable,
    WaitingForInterrupt,
}
