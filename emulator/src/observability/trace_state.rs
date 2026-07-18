//! Machine-level trace configuration and counters.

use std::env;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TraceOptions {
    pub faults: bool,
    pub syscall_dispatch: bool,
    pub writev: bool,
    pub rwsem: bool,
    pub undecoded: bool,
    pub el0_undecoded: bool,
    pub el0_faults: bool,
    pub el0_fault_raw: bool,
    pub bpf: bool,
    pub stack_chk: bool,
    pub mprotect_loop: bool,
    pub fp_traps: bool,
    pub syscall_paths: bool,
    pub exec: bool,
    pub chase_assert: bool,
    pub path_extend: bool,
    pub pc_range: bool,
    pub progress: bool,
}

impl TraceOptions {
    pub(crate) fn from_env() -> Self {
        Self {
            faults: env_flag("WEBBOXVM_TRACE_FAULTS"),
            syscall_dispatch: env_flag("WEBBOXVM_TRACE_SYSCALL_DISPATCH"),
            writev: env_flag("WEBBOXVM_TRACE_WRITEV"),
            rwsem: env_flag("WEBBOXVM_TRACE_RWSEM"),
            undecoded: env_flag("WEBBOXVM_TRACE_UNDECODED"),
            el0_undecoded: env_flag("WEBBOXVM_TRACE_EL0_UNDECODED"),
            el0_faults: env_flag("WEBBOXVM_TRACE_EL0_FAULTS"),
            el0_fault_raw: env_flag("WEBBOXVM_TRACE_EL0_FAULT_RAW"),
            bpf: env_flag("WEBBOXVM_TRACE_BPF"),
            stack_chk: env_flag("WEBBOXVM_TRACE_STACK_CHK"),
            mprotect_loop: env_flag("WEBBOXVM_TRACE_MPROTECT_LOOP"),
            fp_traps: env_flag("WEBBOXVM_TRACE_FP_TRAPS"),
            syscall_paths: env_flag("WEBBOXVM_TRACE_SYSCALL_PATHS"),
            exec: env_flag("WEBBOXVM_TRACE_EXEC"),
            chase_assert: env_flag("WEBBOXVM_TRACE_CHASE_ASSERT"),
            path_extend: env_flag("WEBBOXVM_TRACE_PATH_EXTEND"),
            pc_range: env_flag("WEBBOXVM_TRACE_PC_RANGE"),
            progress: env_flag("WEBBOXVM_TRACE_PROGRESS"),
        }
    }

    pub(crate) const fn has_fetch_hooks(self) -> bool {
        self.chase_assert || self.path_extend || self.undecoded || self.el0_undecoded
    }

    pub(crate) const fn has_instruction_hooks(self) -> bool {
        self.syscall_dispatch
            || self.writev
            || self.syscall_paths
            || self.exec
            || self.stack_chk
            || self.rwsem
            || self.bpf
            || self.mprotect_loop
            || self.pc_range
    }

    pub(crate) const fn has_syscall_return_hooks(self) -> bool {
        self.syscall_paths || self.exec
    }

    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub(crate) const fn allows_parallel_execution(self) -> bool {
        !self.faults
            && !self.has_fetch_hooks()
            && !self.has_instruction_hooks()
            && !self.has_syscall_return_hooks()
            && !self.el0_faults
            && !self.el0_fault_raw
            && !self.fp_traps
            && !self.progress
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TraceSyscall {
    pub nr: u64,
    pub args: [u64; 6],
    pub pc: u64,
    pub step: u64,
}

#[derive(Debug, Default)]
pub(crate) struct TraceCounters {
    pub rwsem: u64,
    pub undecoded: u64,
    pub el0_fault_raw: u64,
    pub fp_simd_trap: u64,
    pub syscall_path: u64,
    pub exec: u64,
    pub chase_assert: u64,
    pub path_extend: u64,
    pub pc_range: u64,
}

#[derive(Debug)]
pub(crate) struct TraceState {
    pub options: TraceOptions,
    pub pending_syscalls: Vec<Option<TraceSyscall>>,
    pub counters: TraceCounters,
}

impl TraceState {
    pub(crate) fn new(num_cores: usize, options: TraceOptions) -> Self {
        Self {
            options,
            pending_syscalls: vec![None; num_cores],
            counters: TraceCounters::default(),
        }
    }
}

fn env_flag(name: &str) -> bool {
    env::var_os(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::TraceOptions;

    #[test]
    fn default_options_have_no_hot_loop_hooks() {
        let options = TraceOptions::default();
        assert!(!options.has_fetch_hooks() && !options.has_instruction_hooks());
        assert!(!options.has_syscall_return_hooks());
        assert!(!options.progress);
    }

    #[test]
    fn hook_groups_report_enabled_options() {
        assert!(
            TraceOptions {
                undecoded: true,
                ..TraceOptions::default()
            }
            .has_fetch_hooks()
        );
        assert!(
            TraceOptions {
                bpf: true,
                ..TraceOptions::default()
            }
            .has_instruction_hooks()
        );
        assert!(
            TraceOptions {
                pc_range: true,
                ..TraceOptions::default()
            }
            .has_instruction_hooks()
        );
        assert!(
            TraceOptions {
                exec: true,
                ..TraceOptions::default()
            }
            .has_syscall_return_hooks()
        );
    }

    #[test]
    fn progress_trace_is_not_a_hot_loop_hook_group() {
        let options = TraceOptions {
            progress: true,
            ..TraceOptions::default()
        };
        assert!(!options.has_fetch_hooks());
        assert!(!options.has_instruction_hooks());
        assert!(!options.has_syscall_return_hooks() && options.progress);
    }
}
