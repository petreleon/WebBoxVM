use super::*;

pub(in crate::arm64::execute) fn exec_sve_zip(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let pairs = elements / 2;
    let start = if instr.op == Opcode::SveZip2 {
        pairs
    } else {
        0
    };
    let left = sve_read_z(cpu, instr.rn as usize);
    let right = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for pair in 0..pairs {
        copy_element(&mut result, pair * 2, &left, start + pair, element_size);
        copy_element(
            &mut result,
            pair * 2 + 1,
            &right,
            start + pair,
            element_size,
        );
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn copy_element(
    dest: &mut [u8; 256],
    dest_element: usize,
    src: &[u8; 256],
    src_element: usize,
    element_size: usize,
) {
    let dest_offset = dest_element * element_size;
    let src_offset = src_element * element_size;
    dest[dest_offset..dest_offset + element_size]
        .copy_from_slice(&src[src_offset..src_offset + element_size]);
}
