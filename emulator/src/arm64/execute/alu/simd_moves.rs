use super::*;

pub(in crate::arm64::execute) fn exec_simd_moves(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;

    match instr.op {
        Opcode::SimdDupByte => {
            let element_size = if instr.cond == 0 {
                1
            } else {
                instr.cond as usize
            };
            let value = read_reg(cpu, instr.rn, element_size == 8) as u128;
            cpu.simd[rd] = simd_replicate_element(value, element_size, instr.size as usize);
        }
        Opcode::SimdDupElem => {
            let element_size = instr.cond.max(1) as usize;
            let value = simd_element(cpu.simd[rn], instr.imm as usize, element_size);
            cpu.simd[rd] = simd_replicate_element(value, element_size, instr.size as usize);
        }
        Opcode::SimdFmovReg64 => {
            cpu.simd[rd] = read_fp_bits(cpu, instr.rn, instr.size) as u128;
        }
        Opcode::SimdFmovGprToD => {
            cpu.simd[rd] = read_reg(cpu, instr.rn, true) as u128;
        }
        Opcode::SimdFmovGprToS => {
            let value = read_reg(cpu, instr.rn, instr.size == 8) as u128;
            cpu.simd[rd] = value & simd_element_mask(instr.size as usize);
        }
        Opcode::SimdFmovDToGpr => {
            write_reg(cpu, instr.rd, cpu.simd[rn] as u64, true);
        }
        Opcode::SimdFmovSToGpr => {
            let value = (cpu.simd[rn] & simd_element_mask(instr.size as usize)) as u64;
            write_reg(cpu, instr.rd, value, instr.size == 8);
        }
        Opcode::SimdFmovLaneToGpr => {
            let shift = (instr.imm as u32) * 64;
            write_reg(cpu, instr.rd, (cpu.simd[rn] >> shift) as u64, true);
        }
        Opcode::SimdFmovImm => {
            let element_size = instr.cond.max(1);
            let value = fp_expand_imm(instr.imm as u8, element_size) as u128;
            cpu.simd[rd] =
                simd_replicate_element(value, element_size as usize, instr.size as usize);
        }
        Opcode::SimdUmov => {
            let element_size = instr.cond.max(1) as u32;
            let shift = (instr.imm as u32) * element_size * 8;
            let bits = element_size * 8;
            let mask = if bits == 64 {
                u64::MAX as u128
            } else {
                (1u128 << bits) - 1
            };
            let value = ((cpu.simd[rn] >> shift) & mask) as u64;
            write_reg(cpu, instr.rd, value, instr.sf);
        }
        Opcode::SimdSmov => {
            let element_size = instr.cond.max(1) as usize;
            let value = simd_signed_element(cpu.simd[rn], instr.imm as usize, element_size);
            if instr.sf {
                write_reg(cpu, instr.rd, value as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        Opcode::SimdInsGprLane => {
            let element_size = instr.cond.max(1) as usize;
            let shift = (instr.imm as usize) * element_size * 8;
            let mask = simd_element_mask(element_size) << shift;
            let source_is_64_bit = element_size == 8;
            let value = (read_reg(cpu, instr.rn, source_is_64_bit) as u128) << shift;
            cpu.simd[rd] = (cpu.simd[rd] & !mask) | (value & mask);
        }
        _ => unreachable!(),
    }
}
