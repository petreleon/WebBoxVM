use super::super::alu::f16_to_f32;
use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 4];

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&lhs, element, element_size);
            let right = sve_element(&rhs, element, element_size);
            let bit = fp_compare(instr.op, left, right, element_size);
            set_predicate_bit(&mut result, element * element_size, bit);
        }
    }

    let flags = sve_pred_test(&mask, &result, element_size, vl_bytes);
    cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    cpu.sve_pred[instr.rd as usize] = result;
}

fn fp_compare(op: Opcode, left: u64, right: u64, element_size: usize) -> bool {
    match element_size {
        2 => compare_f32(op, f16_to_f32(left as u16), f16_to_f32(right as u16)),
        4 => compare_f32(
            op,
            f32::from_bits(left as u32),
            f32::from_bits(right as u32),
        ),
        8 => compare_f64(op, f64::from_bits(left), f64::from_bits(right)),
        _ => false,
    }
}

fn compare_f32(op: Opcode, left: f32, right: f32) -> bool {
    match op {
        Opcode::SveFpFacge => left.abs() >= right.abs(),
        Opcode::SveFpFacgt => left.abs() > right.abs(),
        _ => unreachable!(),
    }
}

fn compare_f64(op: Opcode, left: f64, right: f64) -> bool {
    match op {
        Opcode::SveFpFacge => left.abs() >= right.abs(),
        Opcode::SveFpFacgt => left.abs() > right.abs(),
        _ => unreachable!(),
    }
}
