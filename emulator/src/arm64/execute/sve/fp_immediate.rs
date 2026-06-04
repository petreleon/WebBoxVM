use super::super::alu::fp_expand_imm;
use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_dup_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let value = fp_expand_imm(instr.imm as u8, instr.size);
    let mut result = [0; 256];

    for element in 0..elements {
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}
