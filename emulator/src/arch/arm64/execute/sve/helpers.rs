use super::*;

pub(in crate::arch::arm64::execute) fn predicate_element(
    pred: &[u64; 4],
    element: usize,
    element_size: usize,
) -> bool {
    predicate_bit(pred, element * element_size)
}

pub(in crate::arch::arm64::execute) fn set_predicate_bit(
    pred: &mut [u64; 4],
    bit: usize,
    value: bool,
) {
    if bit >= 256 {
        return;
    }
    let word = bit / 64;
    let offset = bit % 64;
    if value {
        pred[word] |= 1 << offset;
    } else {
        pred[word] &= !(1 << offset);
    }
}

pub(in crate::arch::arm64::execute) fn predicate_bit(pred: &[u64; 4], bit: usize) -> bool {
    bit < 256 && (pred[bit / 64] & (1 << (bit % 64))) != 0
}

pub(in crate::arch::arm64::execute) fn sve_read_z(cpu: &mut Armv8Cpu, reg: usize) -> [u8; 256] {
    sync_z_from_simd(cpu, reg);
    cpu.sve_z[reg]
}

pub(in crate::arch::arm64::execute) fn sve_write_z(
    cpu: &mut Armv8Cpu,
    reg: usize,
    mut value: [u8; 256],
) {
    let vl_bytes = sve_vl_bytes(cpu);
    value[vl_bytes..].fill(0);
    cpu.sve_z[reg] = value;
    sync_simd_from_z(cpu, reg);
}

pub(in crate::arch::arm64::execute) fn sync_z_from_simd(cpu: &mut Armv8Cpu, reg: usize) {
    cpu.sve_z[reg][..16].copy_from_slice(&cpu.simd[reg].to_le_bytes());
}

pub(in crate::arch::arm64::execute) fn sync_simd_from_z(cpu: &mut Armv8Cpu, reg: usize) {
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&cpu.sve_z[reg][..16]);
    cpu.simd[reg] = u128::from_le_bytes(bytes);
}

pub(in crate::arch::arm64::execute) fn copy_sve_element(
    dest: &mut [u8; 256],
    src: &[u8; 256],
    element: usize,
    element_size: usize,
) {
    let offset = element * element_size;
    dest[offset..offset + element_size].copy_from_slice(&src[offset..offset + element_size]);
}

pub(in crate::arch::arm64::execute) fn sve_element(
    vec: &[u8; 256],
    element: usize,
    element_size: usize,
) -> u64 {
    let offset = element * element_size;
    let mut bytes = [0; 8];
    bytes[..element_size].copy_from_slice(&vec[offset..offset + element_size]);
    u64::from_le_bytes(bytes)
}

pub(in crate::arch::arm64::execute) fn sve_set_element(
    vec: &mut [u8; 256],
    element: usize,
    element_size: usize,
    value: u64,
) {
    let offset = element * element_size;
    vec[offset..offset + element_size].copy_from_slice(&value.to_le_bytes()[..element_size]);
}

pub(in crate::arch::arm64::execute) fn sve_element_mask(element_size: usize) -> u64 {
    match element_size {
        1 => u8::MAX as u64,
        2 => u16::MAX as u64,
        4 => u32::MAX as u64,
        8 => u64::MAX,
        _ => 0,
    }
}

pub(in crate::arch::arm64::execute) fn sve_vl_bytes(cpu: &Armv8Cpu) -> usize {
    (cpu.sve_vl_bytes as usize).clamp(16, 256)
}

pub(in crate::arch::arm64::execute) fn sve_pl_bytes(cpu: &Armv8Cpu) -> usize {
    sve_vl_bytes(cpu) / 8
}
