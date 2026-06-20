use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_reverse(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let chunk_size = instr.imm as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            reverse_element(&mut result, &source, element, element_size, chunk_size);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn reverse_element(
    dest: &mut [u8; 256],
    source: &[u8; 256],
    element: usize,
    element_size: usize,
    chunk_size: usize,
) {
    let base = element * element_size;
    let chunks = element_size / chunk_size;
    for chunk in 0..chunks {
        let dst = base + chunk * chunk_size;
        let src = base + (chunks - 1 - chunk) * chunk_size;
        dest[dst..dst + chunk_size].copy_from_slice(&source[src..src + chunk_size]);
    }
}
