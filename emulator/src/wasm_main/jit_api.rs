use super::*;
use crate::arm64::jit::{JIT_STATE_SIZE, compile_wasm64_block_at_pc, hash_raw_words};
use crate::arm64::machine::Machine;
use crate::arm64::translate;
use crate::constants::GIC_SPURIOUS_INTERRUPT;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Pointer to the fixed-layout JIT CPU state inside the main wasm memory.
    ///
    /// Dynamically generated Wasm64 blocks import this same memory and receive
    /// the pointer as `run(state_ptr)`.
    pub fn jit_state_ptr(&mut self) -> u64 {
        self.jit_state.as_mut() as *mut _ as u64
    }

    /// Size of the fixed-layout JIT CPU state in bytes.
    pub fn jit_state_size(&self) -> usize {
        JIT_STATE_SIZE
    }

    /// Last ARM64-to-Wasm64 JIT compile/sync error.
    pub fn jit_last_error(&self) -> String {
        self.jit_last_error.clone()
    }

    /// Guest instructions represented by the last successfully compiled JIT block.
    pub fn jit_last_block_steps(&self) -> usize {
        self.jit_last_block_steps
    }

    /// Start PC of the last successfully compiled JIT block.
    pub fn jit_last_block_start_pc(&self) -> u64 {
        self.jit_last_block_start_pc
    }

    /// Start physical address of the last successfully compiled JIT block.
    pub fn jit_last_block_start_pa(&self) -> u64 {
        self.jit_last_block_start_pa
    }

    /// Exit PC of the last successfully compiled JIT block.
    pub fn jit_last_block_exit_pc(&self) -> u64 {
        self.jit_last_block_exit_pc
    }

    /// Raw-code fingerprint of the last successfully compiled JIT block.
    pub fn jit_last_block_raw_hash(&self) -> u64 {
        self.jit_last_block_raw_hash
    }

    /// Copy one emulated core's architectural register state into the JIT state buffer.
    pub fn jit_sync_state_from_core(&mut self, core_id: Option<usize>) -> bool {
        let core_id = core_id.unwrap_or(0);
        let cpu = if let Some(ref boot) = self.boot {
            boot.machine.cpus.get(core_id)
        } else {
            self.machine.cpus.get(core_id)
        };

        let Some(cpu) = cpu else {
            self.jit_last_error = format!("core {core_id} does not exist");
            return false;
        };

        self.jit_state.copy_from_cpu(cpu);
        self.jit_last_error.clear();
        true
    }

    /// Copy the JIT state buffer back into one emulated core.
    ///
    /// This is intentionally explicit so the browser worker can validate a JIT
    /// block before allowing it to mutate the VM.
    pub fn jit_sync_state_to_core(&mut self, core_id: Option<usize>) -> bool {
        let core_id = core_id.unwrap_or(0);
        let cpu = if let Some(ref mut boot) = self.boot {
            boot.machine.cpus.get_mut(core_id)
        } else {
            self.machine.cpus.get_mut(core_id)
        };

        let Some(cpu) = cpu else {
            self.jit_last_error = format!("core {core_id} does not exist");
            return false;
        };

        self.jit_state.copy_to_cpu(cpu);
        self.jit_last_error.clear();
        true
    }

    /// Compile the block at the selected core's current PC into a Wasm64 module.
    ///
    /// Returns an empty byte vector when the current block must fall back to the
    /// interpreter. Use `jit_last_error()` for the reason.
    pub fn jit_compile_current_block(&mut self, core_id: Option<usize>) -> Vec<u8> {
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref boot) = self.boot {
            let Some(cpu) = boot.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            compile_wasm64_block_at_pc(cpu, &boot.machine.bus)
        } else {
            let Some(cpu) = self.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            compile_wasm64_block_at_pc(cpu, &self.machine.bus)
        };

        match result {
            Ok(module) => {
                self.jit_last_error.clear();
                self.jit_last_block_steps = module.guest_instr_count;
                self.jit_last_block_start_pc = module.start_pc;
                self.jit_last_block_start_pa = module.start_pa;
                self.jit_last_block_exit_pc = module.exit_pc;
                self.jit_last_block_raw_hash = module.raw_hash;
                module.bytes
            }
            Err(err) => {
                self.jit_last_error = err.to_string();
                self.jit_last_block_steps = 0;
                self.jit_last_block_start_pc = 0;
                self.jit_last_block_start_pa = 0;
                self.jit_last_block_exit_pc = 0;
                self.jit_last_block_raw_hash = 0;
                Vec::new()
            }
        }
    }

    /// Validate that a cached JIT block still matches current guest code.
    pub fn jit_validate_block(
        &mut self,
        core_id: Option<usize>,
        start_pc: u64,
        start_pa: u64,
        raw_hash: u64,
        steps: usize,
    ) -> bool {
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref boot) = self.boot {
            validate_jit_block(&boot.machine, core_id, start_pc, start_pa, raw_hash, steps)
        } else {
            validate_jit_block(&self.machine, core_id, start_pc, start_pa, raw_hash, steps)
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }

    /// Commit the JIT state buffer back to a core after a generated block runs.
    ///
    /// This is deliberately conservative. It only commits for single-core VMs
    /// and rejects blocks that could cross a timer deadline or pending unmasked
    /// IRQ boundary.
    pub fn jit_commit_state_to_core(
        &mut self,
        core_id: Option<usize>,
        steps: usize,
        expected_exit_pc: u64,
    ) -> bool {
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref mut boot) = self.boot {
            commit_jit_state(
                &self.jit_state,
                &mut boot.machine,
                core_id,
                steps,
                expected_exit_pc,
            )
        } else {
            commit_jit_state(
                &self.jit_state,
                &mut self.machine,
                core_id,
                steps,
                expected_exit_pc,
            )
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }
}

fn validate_jit_block(
    machine: &Machine,
    core_id: usize,
    start_pc: u64,
    start_pa: u64,
    raw_hash: u64,
    steps: usize,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot validate an empty JIT block".to_string());
    }
    let Some(cpu) = machine.cpus.get(core_id) else {
        return Err(format!("core {core_id} does not exist"));
    };
    if cpu.regs.pc != start_pc {
        return Err(format!(
            "cached JIT block starts at 0x{start_pc:016x}, current PC is 0x{:016x}",
            cpu.regs.pc
        ));
    }

    let mut tlb = cpu.tlb.clone();
    let current_pa = translate(&cpu.sys, &mut tlb, &machine.bus.mem, start_pc)
        .map_err(|_| "JIT block start translation fault".to_string())?;
    if current_pa != start_pa {
        return Err(format!(
            "cached JIT block PA changed: cached=0x{start_pa:016x} current=0x{current_pa:016x}"
        ));
    }

    for index in 1..steps {
        let pc = start_pc + index as u64 * 4;
        let addr = start_pa + index as u64 * 4;
        let current_pa = translate(&cpu.sys, &mut tlb, &machine.bus.mem, pc)
            .map_err(|_| format!("cached JIT block PC 0x{pc:016x} translation fault"))?;
        if current_pa != addr {
            return Err(format!(
                "cached JIT block PA changed at PC 0x{pc:016x}: cached=0x{addr:016x} current=0x{current_pa:016x}"
            ));
        }
    }

    let mut raw_words = Vec::with_capacity(steps);
    for index in 0..steps {
        let addr = start_pa + index as u64 * 4;
        let raw = machine
            .bus
            .mem
            .read(addr, 4)
            .ok_or_else(|| format!("cached JIT block word at 0x{addr:016x} is unreadable"))?;
        raw_words.push(raw as u32);
    }

    let current_hash = hash_raw_words(start_pa, raw_words);
    if current_hash != raw_hash {
        return Err(format!(
            "cached JIT block raw hash changed: cached=0x{raw_hash:016x} current=0x{current_hash:016x}"
        ));
    }

    Ok(())
}

fn commit_jit_state(
    state: &crate::arm64::jit::WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot commit an empty JIT block".to_string());
    }
    if state.pc != expected_exit_pc {
        return Err(format!(
            "JIT block exit mismatch: expected=0x{expected_exit_pc:016x} actual=0x{:016x}",
            state.pc
        ));
    }
    if machine.cpus.len() != 1 {
        return Err("JIT commit is currently restricted to single-core VMs".to_string());
    }
    if machine.active_core != core_id {
        return Err(format!(
            "JIT core mismatch: active core is {}, requested {core_id}",
            machine.active_core
        ));
    }

    let steps = steps as u64;
    {
        let Some(cpu) = machine.cpus.get(core_id) else {
            return Err(format!("core {core_id} does not exist"));
        };

        if let Some(deadline) = cpu.sys.next_timer_deadline() {
            let end_cycle = cpu.sys.cycle_count.saturating_add(steps);
            if deadline <= end_cycle {
                return Err(format!(
                    "JIT block crosses timer deadline at cycle {deadline}"
                ));
            }
        }

        let external_irq = machine.bus.gic.next_pending_enabled();
        let cpu_irq = cpu.sys.irq_pending && cpu.sys.last_irq_id != GIC_SPURIOUS_INTERRUPT as u32;
        if !cpu.pstate.irq_masked() && (cpu_irq || external_irq.is_some()) {
            return Err("JIT block crosses an unmasked pending IRQ boundary".to_string());
        }
    }

    let cpu = &mut machine.cpus[core_id];
    let cycle_count = cpu.sys.cycle_count;
    state.copy_to_cpu(cpu);
    cpu.sys.cycle_count = cycle_count.wrapping_add(steps);
    machine.total_steps = machine.total_steps.wrapping_add(steps);
    machine.active_core = (core_id + 1) % machine.cpus.len();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        DESC_AF_BIT, DESC_BLOCK, DESC_TABLE, RAM_BASE, SCTLR_MMU_ENABLE, TCR_T1SZ_SHIFT,
    };

    const NOP: u32 = 0xd503_201f;

    fn map_two_ttbr0_pages(machine: &mut Machine, page0_pa: u64, page1_pa: u64) {
        let l1_table = RAM_BASE;
        let l2_table = RAM_BASE + 0x1000;
        let l3_table = RAM_BASE + 0x2000;

        machine.bus.mem.write(l1_table, 8, l2_table | DESC_TABLE);
        machine.bus.mem.write(l2_table, 8, l3_table | DESC_TABLE);
        machine
            .bus
            .mem
            .write(l3_table, 8, page0_pa | DESC_AF_BIT | DESC_BLOCK);
        machine
            .bus
            .mem
            .write(l3_table + 8, 8, page1_pa | DESC_AF_BIT | DESC_BLOCK);

        let cpu = &mut machine.cpus[0];
        cpu.sys.ttbr0_el1 = l1_table;
        cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
        cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    }

    #[test]
    fn validate_jit_block_rejects_changed_second_instruction_translation() {
        let mut machine = Machine::new(1);
        let start_pc = 0xffc;
        let start_pa = RAM_BASE + 0x3ffc;
        map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
        machine.cpus[0].regs.pc = start_pc;
        machine.bus.mem.write(start_pa, 4, NOP as u64);
        machine.bus.mem.write(start_pa + 4, 4, NOP as u64);

        let hash = hash_raw_words(start_pa, [NOP, NOP]);
        let err = validate_jit_block(&machine, 0, start_pc, start_pa, hash, 2)
            .expect_err("non-contiguous second instruction mapping must be rejected");

        assert!(
            err.contains("cached JIT block PA changed at PC 0x0000000000001000"),
            "{err}"
        );
    }
}
