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
    let is_store = matches!(instr.op, Opcode::Stp | Opcode::MteStgp | Opcode::SimdStp);
    let pa1 =
        translate_or_data_fault(cpu, &mut bus.mem, va, is_store, "LDP/STP translation fault")?;
    let pa2 = translate_or_data_fault(
        cpu,
        &mut bus.mem,
        va + size,
        is_store,
        "LDP/STP translation fault",
    )?;

    match instr.op {
        Opcode::Ldp => {
            let lo = read_guest(cpu, bus, va, size as u8, "LDP bus fault")?;
            let hi = read_guest(cpu, bus, va + size, size as u8, "LDP bus fault")?;
            trace_syscall_frame_access(cpu, &instr, "LDP.0", va, pa1, size as u8, Some(lo));
            trace_syscall_frame_access(cpu, &instr, "LDP.1", va + size, pa2, size as u8, Some(hi));
            write_reg(cpu, instr.rd, lo, instr.sf);
            write_reg(cpu, instr.rm, hi, instr.sf);
        }
        Opcode::Ldpsw => {
            let lo = read_guest(cpu, bus, va, 4, "LDPSW bus fault")? as u32 as i32 as i64 as u64;
            let hi =
                read_guest(cpu, bus, va + 4, 4, "LDPSW bus fault")? as u32 as i32 as i64 as u64;
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
            write_guest(cpu, bus, va, access_size, val1, fault_label)?;
            trace_text_store(cpu, bus, &instr, label1, va + size, pa2, access_size, val2);
            write_guest(cpu, bus, va + size, access_size, val2, fault_label)?;
        }
        Opcode::SimdLdp => {
            let access_size = size as u8;
            cpu.simd[instr.rd as usize] =
                read_simd_guest(cpu, bus, va, access_size, "SIMD LDP bus fault")?;
            cpu.simd[instr.rm as usize] =
                read_simd_guest(cpu, bus, va + size, access_size, "SIMD LDP bus fault")?;
        }
        Opcode::SimdStp => {
            let access_size = size as u8;
            write_simd_guest(
                cpu,
                bus,
                va,
                access_size,
                cpu.simd[instr.rd as usize],
                "SIMD STP bus fault",
            )?;
            write_simd_guest(
                cpu,
                bus,
                va + size,
                access_size,
                cpu.simd[instr.rm as usize],
                "SIMD STP bus fault",
            )?;
        }
        _ => unreachable!(),
    }
    if new_base != base {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}
