use super::pair_bulk::{read_pair_scalars, read_pair_simd, write_pair_scalars, write_pair_simd};
use super::*;

pub(in crate::arch::arm64::execute) fn exec_ldp_stp(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 {
        instr.size as u64
    } else if instr.sf {
        8u64
    } else {
        4u64
    };
    let (va, new_base) = match instr.cond {
        1 => (base, branch_target(base, instr.imm)),
        3 => {
            let b = branch_target(base, instr.imm);
            (b, b)
        }
        _ => (branch_target(base, instr.imm), base),
    };
    if instr.op == Opcode::MteStgp && va % 16 != 0 {
        return Err("MTE tag granule alignment fault");
    }
    match instr.op {
        Opcode::Ldp => {
            let access_size = size as u8;
            let (pa1, pa2) = translate_pair(cpu, bus, va, size, false, "LDP bus fault")?;
            let (lo, hi) =
                read_pair_scalars(cpu, bus, va, (pa1, pa2), access_size, "LDP bus fault")?;
            trace_syscall_frame_access(cpu, &instr, "LDP.0", va, pa1, size as u8, Some(lo));
            trace_syscall_frame_access(cpu, &instr, "LDP.1", va + size, pa2, size as u8, Some(hi));
            write_reg(cpu, instr.rd, lo, instr.sf);
            write_reg(cpu, instr.rm, hi, instr.sf);
        }
        Opcode::Ldpsw => {
            let (pa1, pa2) = translate_pair(cpu, bus, va, 4, false, "LDPSW bus fault")?;
            let (lo, hi) = read_pair_scalars(cpu, bus, va, (pa1, pa2), 4, "LDPSW bus fault")?;
            let lo = lo as u32 as i32 as i64 as u64;
            let hi = hi as u32 as i32 as i64 as u64;
            trace_syscall_frame_access(cpu, &instr, "LDPSW.0", va, pa1, 4, Some(lo));
            trace_syscall_frame_access(cpu, &instr, "LDPSW.1", va + 4, pa2, 4, Some(hi));
            write_reg(cpu, instr.rd, lo, true);
            write_reg(cpu, instr.rm, hi, true);
        }
        Opcode::Stp | Opcode::MteStgp => {
            let access_size = size as u8;
            let val1 = read_reg(cpu, instr.rd, instr.sf);
            let val2 = read_reg(cpu, instr.rm, instr.sf);
            let (label0, label1) = if instr.op == Opcode::MteStgp {
                ("STGP.0", "STGP.1")
            } else {
                ("STP.0", "STP.1")
            };
            let fault_label = if instr.op == Opcode::MteStgp {
                "STGP bus fault"
            } else {
                "STP bus fault"
            };
            let (pa1, pa2) = translate_pair(cpu, bus, va, size, true, fault_label)?;
            trace_syscall_frame_access(cpu, &instr, label0, va, pa1, access_size, Some(val1));
            trace_syscall_frame_access(
                cpu,
                &instr,
                label1,
                va + size,
                pa2,
                access_size,
                Some(val2),
            );
            trace_text_store(cpu, bus, &instr, label0, va, pa1, access_size, val1);
            trace_text_store(cpu, bus, &instr, label1, va + size, pa2, access_size, val2);
            if !access_crosses_page(va, access_size * 2) {
                write_pair_scalars(cpu, bus, va, pa1, access_size, val1, val2, fault_label)?;
            } else {
                write_guest_translated(cpu, bus, va, pa1, access_size, val1, fault_label)?;
                write_guest_translated(cpu, bus, va + size, pa2, access_size, val2, fault_label)?;
            }
        }
        Opcode::SimdLdp => {
            let access_size = size as u8;
            let (pa1, pa2) = translate_pair(cpu, bus, va, size, false, "SIMD LDP bus fault")?;
            let (lo, hi) =
                read_pair_simd(cpu, bus, va, (pa1, pa2), access_size, "SIMD LDP bus fault")?;
            cpu.simd[instr.rd as usize] = lo;
            cpu.simd[instr.rm as usize] = hi;
        }
        Opcode::SimdStp => {
            let access_size = size as u8;
            let (pa1, pa2) = translate_pair(cpu, bus, va, size, true, "SIMD STP bus fault")?;
            if !write_pair_simd(
                cpu,
                bus,
                va,
                pa1,
                access_size,
                cpu.simd[instr.rd as usize],
                cpu.simd[instr.rm as usize],
                "SIMD STP bus fault",
            )? {
                write_simd_guest_translated(
                    cpu,
                    bus,
                    va,
                    pa1,
                    access_size,
                    cpu.simd[instr.rd as usize],
                    "SIMD STP bus fault",
                )?;
                write_simd_guest_translated(
                    cpu,
                    bus,
                    va + size,
                    pa2,
                    access_size,
                    cpu.simd[instr.rm as usize],
                    "SIMD STP bus fault",
                )?;
            }
        }
        _ => unreachable!(),
    }
    if new_base != base {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn translate_pair(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u64,
    write: bool,
    fault_label: &'static str,
) -> Result<(u64, u64), &'static str> {
    let pa1 = translate_or_data_fault(cpu, &mut bus.mem, va, write, fault_label)?;
    let pair_bytes = size * 2;
    if pair_bytes <= u8::MAX as u64 && !access_crosses_page(va, pair_bytes as u8) {
        return Ok((pa1, pa1 + size));
    }
    let pa2 = translate_or_data_fault(cpu, &mut bus.mem, va + size, write, fault_label)?;
    Ok((pa1, pa2))
}
