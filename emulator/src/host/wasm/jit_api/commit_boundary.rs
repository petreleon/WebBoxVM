use crate::arch::arm64::CpuLifecycle;
use crate::constants::GIC_SPURIOUS_INTERRUPT;
use crate::runtime::Machine;

pub(super) fn can_commit_jit_block_now(
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
) -> Result<(), String> {
    validate_jit_commit_target(machine, core_id, steps)?;
    let cpu = &machine.cpus[core_id];
    let steps = steps as u64;
    if let Some(deadline) = cpu.sys.next_timer_deadline() {
        let end_cycle = cpu.sys.cycle_count.saturating_add(steps);
        if deadline < end_cycle {
            return Err(format!(
                "JIT block crosses timer deadline at cycle {deadline} end={end_cycle}"
            ));
        }
    }

    reject_skipped_remote_timer(machine, core_id, steps)?;
    if cpu.pstate.irq_masked() {
        return Ok(());
    }
    machine.bus.refresh_interrupts();
    let external_irq = machine.bus.gic.next_pending_enabled_for_cpu(core_id);
    let cpu_irq = cpu.sys.irq_pending && cpu.sys.last_irq_id != GIC_SPURIOUS_INTERRUPT as u32;
    if cpu_irq || external_irq.is_some() {
        return Err("JIT block crosses an unmasked pending IRQ boundary".to_string());
    }
    Ok(())
}

pub(super) fn validate_jit_commit_target(
    machine: &Machine,
    core_id: usize,
    steps: usize,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot commit an empty JIT block".to_string());
    }
    let Some(cpu) = machine.cpus.get(core_id) else {
        return Err(format!("core {core_id} does not exist"));
    };
    if machine.active_core != core_id {
        return Err(format!(
            "JIT core mismatch: active core is {}, requested {core_id}",
            machine.active_core
        ));
    }
    if cpu.lifecycle != CpuLifecycle::Runnable {
        return Err(format!("JIT core {core_id} is not runnable"));
    }
    if machine.cpus.len() > 1 && cpu.sys.cycle_count != machine.virtual_time {
        return Err(format!(
            "JIT core {core_id} was not scheduler-prepared: core cycle={} virtual time={}",
            cpu.sys.cycle_count, machine.virtual_time
        ));
    }
    Ok(())
}

fn reject_skipped_remote_timer(
    machine: &Machine,
    core_id: usize,
    steps: u64,
) -> Result<(), String> {
    let end = predicted_jit_end_time(machine, core_id, steps);
    let skipped = machine
        .cpus
        .iter()
        .enumerate()
        .filter(|(core, cpu)| *core != core_id && cpu.lifecycle != CpuLifecycle::PoweredOff)
        .filter_map(|(core, cpu)| {
            cpu.sys
                .next_timer_deadline()
                .map(|deadline| (core, deadline))
        })
        // Equality is safe: the normal scheduler wakes the core on the next
        // prepare after the quantum lands exactly on the timer boundary.
        .filter(|(_, deadline)| *deadline < end)
        .min_by_key(|(_, deadline)| *deadline);
    let Some((core, deadline)) = skipped else {
        return Ok(());
    };
    Err(format!(
        "JIT block crosses core {core} remote timer deadline at cycle {deadline} end={end}"
    ))
}

fn predicted_jit_end_time(machine: &Machine, core_id: usize, steps: u64) -> u64 {
    machine
        .virtual_time
        .saturating_add(steps)
        .max(machine.cpus[core_id].sys.cycle_count.saturating_add(steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicted_end_time_saturates_instead_of_wrapping() {
        let mut machine = Machine::new(1);
        machine.cpus[0].sys.cycle_count = u64::MAX - 1;

        assert_eq!(predicted_jit_end_time(&machine, 0, 2), u64::MAX);
    }
}
