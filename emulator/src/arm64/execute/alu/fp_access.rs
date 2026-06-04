use super::*;

pub(in crate::arm64::execute) fn read_fp_bits(cpu: &Armv8Cpu, reg: u8, size: u8) -> u64 {
    match size {
        2 => (cpu.simd[reg as usize] & u16::MAX as u128) as u64,
        4 => (cpu.simd[reg as usize] & u32::MAX as u128) as u64,
        _ => cpu.simd[reg as usize] as u64,
    }
}

pub(in crate::arm64::execute) fn write_fp_bits(cpu: &mut Armv8Cpu, reg: u8, bits: u64, size: u8) {
    cpu.simd[reg as usize] = match size {
        2 => (bits as u16) as u128,
        4 => (bits as u32) as u128,
        _ => bits as u128,
    };
}

pub(in crate::arm64::execute) fn read_fp_as_f64(cpu: &Armv8Cpu, reg: u8, size: u8) -> f64 {
    match size {
        2 => f16_to_f32(read_fp_bits(cpu, reg, 2) as u16) as f64,
        4 => f32::from_bits(read_fp_bits(cpu, reg, 4) as u32) as f64,
        _ => f64::from_bits(read_fp_bits(cpu, reg, 8)),
    }
}
