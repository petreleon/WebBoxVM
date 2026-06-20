use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_xar(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let left = sve_read_z(cpu, instr.rn as usize);
    let right = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let value =
            sve_element(&left, element, element_size) ^ sve_element(&right, element, element_size);
        sve_set_element(
            &mut result,
            element,
            element_size,
            rotate_right(value, instr.imm as u32, element_size),
        );
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn rotate_right(value: u64, shift: u32, element_size: usize) -> u64 {
    let bits = (element_size * 8) as u32;
    let mask = sve_element_mask(element_size);
    let shift = shift % bits;
    let value = value & mask;
    if shift == 0 {
        value
    } else {
        ((value >> shift) | (value << (bits - shift))) & mask
    }
}
