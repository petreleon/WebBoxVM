use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_logical_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let elements = sve_vl_bytes(cpu) / 8;
    let source = sve_read_z(cpu, instr.rd as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&source, element, 8);
        let value = match instr.op {
            Opcode::SveAndImm => left & instr.imm,
            Opcode::SveOrrImm => left | instr.imm,
            Opcode::SveEorImm => left ^ instr.imm,
            Opcode::SveDupm => instr.imm,
            _ => unreachable!(),
        };
        sve_set_element(&mut result, element, 8, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}
