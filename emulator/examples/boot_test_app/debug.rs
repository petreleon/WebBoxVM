use emulator::boot::BootContext;
use std::env;

pub(super) fn print_debug_state(ctx: &BootContext) {
    if env::var_os("BOOT_TEST_DEBUG_STATE").is_none() {
        return;
    }

    let cpu = ctx.machine.core(0);
    println!(
        "CPU: cycle={} pstate=0x{:x} irq_pending={} last_irq={} cntp_ctl=0x{:x} cntp_cval={} cntv_ctl=0x{:x} cntv_cval={} elr=0x{:x} esr=0x{:x} far=0x{:x}",
        cpu.sys.cycle_count,
        cpu.pstate.to_u64(),
        cpu.sys.irq_pending,
        cpu.sys.last_irq_id,
        cpu.sys.cntp_ctl_el0,
        cpu.sys.cntp_cval_el0,
        cpu.sys.cntv_ctl_el0,
        cpu.sys.cntv_cval_el0,
        cpu.sys.elr_el1,
        cpu.sys.esr_el1,
        cpu.sys.far_el1,
    );
    println!(
        "GIC: enable0=0x{:08x} enable1=0x{:08x} pending0=0x{:08x} pending1=0x{:08x} group0=0x{:08x} group1=0x{:08x}",
        ctx.machine.bus.gic.enable_word(0),
        ctx.machine.bus.gic.enable_word(1),
        ctx.machine.bus.gic.pending_word(0),
        ctx.machine.bus.gic.pending_word(1),
        ctx.machine.bus.gic.group[0],
        ctx.machine.bus.gic.group[1],
    );
}
