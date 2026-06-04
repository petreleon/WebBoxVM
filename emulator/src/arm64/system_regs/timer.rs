pub(in crate::arm64::system_regs) fn timer_cval_from_tval(cycle_count: u64, tval: u64) -> u64 {
    let delta = tval as u32 as i32 as i64;
    cycle_count.wrapping_add(delta as u64)
}

pub(in crate::arm64::system_regs) fn timer_tval(cval: u64, cycle_count: u64) -> u64 {
    let remaining = (cval as i64).wrapping_sub(cycle_count as i64) as i32;
    remaining as u32 as u64
}
