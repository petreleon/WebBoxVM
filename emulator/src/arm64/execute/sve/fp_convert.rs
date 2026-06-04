use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_convert(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let value = sve_element(&source, element, element_size);
            let converted = convert_value(instr.op, value, instr.imm as usize, instr.rm as usize);
            sve_set_element(&mut result, element, element_size, converted);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn convert_value(op: Opcode, value: u64, src_size: usize, dst_size: usize) -> u64 {
    match op {
        Opcode::SveScvtf => scvtf_value(value, src_size, dst_size),
        Opcode::SveFcvtzs => fcvtzs_value(value, src_size, dst_size),
        _ => unreachable!(),
    }
}

fn scvtf_value(value: u64, src_size: usize, dst_size: usize) -> u64 {
    match (src_size, dst_size) {
        (4, 4) => ((value as u32 as i32) as f32).to_bits() as u64,
        (4, 8) => ((value as u32 as i32) as f64).to_bits(),
        (8, 4) => ((value as i64) as f32).to_bits() as u64,
        (8, 8) => ((value as i64) as f64).to_bits(),
        _ => unreachable!(),
    }
}

fn fcvtzs_value(value: u64, src_size: usize, dst_size: usize) -> u64 {
    match (src_size, dst_size) {
        (4, 4) => f32::from_bits(value as u32).trunc() as i32 as u32 as u64,
        (4, 8) => f32::from_bits(value as u32).trunc() as i64 as u64,
        (8, 4) => f64::from_bits(value).trunc() as i32 as i64 as u64,
        (8, 8) => f64::from_bits(value).trunc() as i64 as u64,
        _ => unreachable!(),
    }
}
