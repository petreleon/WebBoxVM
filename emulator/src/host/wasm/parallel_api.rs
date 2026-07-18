use super::*;
use crate::runtime::ParallelRunStats;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    pub fn parallel_begin_kernel(&mut self, max_steps: usize) -> Result<u64, JsValue> {
        let start = self.require_parallel_start();
        let machine = self
            .active_machine_mut()
            .ok_or_else(|| JsValue::from_str("no kernel loaded"))?;
        machine
            .begin_parallel_wasm(max_steps, start)
            .map_err(JsValue::from_str)
    }

    pub fn parallel_worker_threads(&self) -> usize {
        let _access = self.require_parallel_idle();
        self.parallel_stats().worker_threads
    }

    pub fn parallel_max_local_in_flight(&self) -> usize {
        let _access = self.require_parallel_idle();
        self.parallel_stats().max_local_in_flight
    }
}

impl Emulator {
    fn active_machine_mut(&mut self) -> Option<&mut Machine> {
        self.boot
            .as_mut()
            .map(|boot| &mut boot.machine)
            .or(Some(self.machine.as_mut()))
    }

    fn parallel_stats(&self) -> ParallelRunStats {
        self.boot.as_ref().map_or_else(
            || self.machine.parallel_run_stats(),
            |boot| boot.machine.parallel_run_stats(),
        )
    }
}

#[wasm_bindgen]
pub fn run_parallel_core(run_token: u64, core: usize) -> Result<(), JsValue> {
    Machine::run_parallel_wasm_core(run_token, core).map_err(JsValue::from_str)
}

#[wasm_bindgen]
pub fn cancel_parallel_run(run_token: u64) -> Result<(), JsValue> {
    Machine::cancel_parallel_wasm(run_token).map_err(JsValue::from_str)
}

#[wasm_bindgen]
pub fn finish_parallel_run(run_token: u64) -> Result<String, JsValue> {
    let (steps, pc) = Machine::finish_parallel_wasm(run_token).map_err(JsValue::from_str)?;
    Ok(format!("KERNEL: {steps} steps, PC={pc:#018x}"))
}
