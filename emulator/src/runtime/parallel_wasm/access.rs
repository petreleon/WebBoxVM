use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const IDLE: u8 = 0;
const ACCESS: u8 = 1;
#[cfg(any(test, target_arch = "wasm64"))]
const PARALLEL: u8 = 2;
#[cfg(any(test, target_arch = "wasm64"))]
const DROPPING: u8 = 3;

pub(crate) const PARALLEL_ACCESS_ERROR: &str =
    "parallel run must finish before accessing emulator state";

pub(crate) struct WasmAccessControl {
    state: AtomicU8,
}

pub(crate) struct WasmIdleAccess {
    control: Arc<WasmAccessControl>,
}

#[cfg(any(test, target_arch = "wasm64"))]
pub(crate) struct WasmParallelStart {
    control: Arc<WasmAccessControl>,
    committed: bool,
}

#[cfg(any(test, target_arch = "wasm64"))]
pub(crate) enum WasmDropAccess {
    Drop,
    Leak,
}

impl WasmAccessControl {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: AtomicU8::new(IDLE),
        })
    }

    pub(crate) fn try_idle(self: &Arc<Self>) -> Result<WasmIdleAccess, &'static str> {
        self.claim_access()?;
        Ok(WasmIdleAccess {
            control: Arc::clone(self),
        })
    }

    #[cfg(any(test, target_arch = "wasm64"))]
    pub(crate) fn try_parallel_start(self: &Arc<Self>) -> Result<WasmParallelStart, &'static str> {
        self.claim_access()?;
        Ok(WasmParallelStart {
            control: Arc::clone(self),
            committed: false,
        })
    }

    #[cfg(any(test, target_arch = "wasm64"))]
    pub(crate) fn finish_parallel(&self) {
        match self
            .state
            .compare_exchange(PARALLEL, IDLE, Ordering::Release, Ordering::Acquire)
        {
            Ok(_) | Err(DROPPING) => {}
            Err(_) => panic!("parallel access state must be active at finalize"),
        }
    }

    #[cfg(any(test, target_arch = "wasm64"))]
    pub(crate) fn claim_drop(&self) -> WasmDropAccess {
        loop {
            let state = self.state.load(Ordering::Acquire);
            match state {
                IDLE => match self.state.compare_exchange(
                    IDLE,
                    DROPPING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return WasmDropAccess::Drop,
                    Err(_) => continue,
                },
                ACCESS => std::hint::spin_loop(),
                PARALLEL => match self.state.compare_exchange(
                    PARALLEL,
                    DROPPING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return WasmDropAccess::Leak,
                    Err(_) => continue,
                },
                DROPPING => return WasmDropAccess::Leak,
                _ => unreachable!("invalid wasm access state"),
            }
        }
    }

    #[cfg(any(test, target_arch = "wasm64"))]
    pub(crate) fn require_parallel_run(&self) -> Result<(), &'static str> {
        match self.state.load(Ordering::Acquire) {
            PARALLEL | DROPPING => Ok(()),
            IDLE | ACCESS => Err("parallel run is not ready"),
            _ => unreachable!("invalid wasm access state"),
        }
    }

    fn claim_access(&self) -> Result<(), &'static str> {
        self.state
            .compare_exchange(IDLE, ACCESS, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| PARALLEL_ACCESS_ERROR)
    }
}

#[cfg(any(test, target_arch = "wasm64"))]
impl WasmParallelStart {
    pub(crate) fn control(&self) -> Arc<WasmAccessControl> {
        Arc::clone(&self.control)
    }

    pub(crate) fn commit(mut self) {
        self.control
            .state
            .compare_exchange(ACCESS, PARALLEL, Ordering::Release, Ordering::Relaxed)
            .expect("parallel start must own emulator access");
        self.committed = true;
    }
}

#[cfg(any(test, target_arch = "wasm64"))]
impl Drop for WasmParallelStart {
    fn drop(&mut self) {
        if !self.committed {
            self.control.state.store(IDLE, Ordering::Release);
        }
    }
}

impl Drop for WasmIdleAccess {
    fn drop(&mut self) {
        self.control.state.store(IDLE, Ordering::Release);
    }
}
