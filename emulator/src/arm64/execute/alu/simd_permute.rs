use super::*;

pub(in crate::arm64::execute) fn exec_simd_permute(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdExt => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let vector_size = instr.size as usize;
            let index = (instr.imm as usize).min(vector_size.saturating_sub(1));
            let mut out = 0u128;
            for lane in 0..vector_size {
                let source_index = index + lane;
                let byte = if source_index < vector_size {
                    simd_byte(lhs, source_index)
                } else {
                    simd_byte(rhs, source_index - vector_size)
                };
                out |= (byte as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdRev64 => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_reverse_elements_in_groups(cpu.simd[rn], element_size, instr.size as usize, 8);
        }
        Opcode::SimdRev32 => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_reverse_elements_in_groups(cpu.simd[rn], element_size, instr.size as usize, 4);
        }
        Opcode::SimdInsElem => {
            let element_size = instr.cond.max(1) as usize;
            let dest_lane = (instr.imm >> 8) as usize;
            let source_lane = (instr.imm & 0xff) as usize;
            let bits = element_size * 8;
            let element_mask = simd_element_mask(element_size);
            let element = simd_element(cpu.simd[rn], source_lane, element_size);
            let dest_mask = element_mask << (dest_lane * bits);
            cpu.simd[rd] =
                (cpu.simd[rd] & !dest_mask) | ((element & element_mask) << (dest_lane * bits));
        }
        Opcode::SimdUzp1 | Opcode::SimdUzp2 => {
            let element_size = instr.imm.max(1) as usize;
            let high_half = instr.op == Opcode::SimdUzp2;
            cpu.simd[rd] = simd_uzp(
                cpu.simd[rn],
                cpu.simd[rm],
                element_size,
                instr.size as usize,
                high_half,
            );
        }
        Opcode::SimdTrn1 | Opcode::SimdTrn2 => {
            cpu.simd[rd] = simd_trn(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                instr.op == Opcode::SimdTrn2,
            );
        }
        Opcode::SimdZip1 | Opcode::SimdZip2 => {
            let element_size = instr.imm.max(1) as usize;
            let high_half = instr.op == Opcode::SimdZip2;
            cpu.simd[rd] = simd_zip(
                cpu.simd[rn],
                cpu.simd[rm],
                element_size,
                instr.size as usize,
                high_half,
            );
        }
        Opcode::SimdTbl => {
            let table_count = instr.cond.max(1) as usize;
            let mut out = 0u128;
            for lane in 0..instr.size as usize {
                let index = simd_byte(cpu.simd[rm], lane) as usize;
                let byte = if index < table_count * 16 {
                    let table_reg = (rn + index / 16) % 32;
                    simd_byte(cpu.simd[table_reg], index % 16)
                } else {
                    0
                };
                out |= (byte as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        _ => unreachable!(),
    }
}
