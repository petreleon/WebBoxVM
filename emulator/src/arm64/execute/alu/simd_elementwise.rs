use super::*;

pub(in crate::arm64::execute) fn simd_elementwise_binary<F>(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    f: F,
) -> u128
where
    F: Fn(u128, u128, u128) -> u128,
{
    let bits = element_size * 8;
    let element_mask = simd_element_mask(element_size);
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let a = simd_element(lhs, lane, element_size);
        let b = simd_element(rhs, lane, element_size);
        out |= (f(a, b, element_mask) & element_mask) << (lane * bits);
    }
    out
}

pub(in crate::arm64::execute) fn simd_elementwise_ternary<F>(
    dst: u128,
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    f: F,
) -> u128
where
    F: Fn(u128, u128, u128) -> u128,
{
    let bits = element_size * 8;
    let element_mask = simd_element_mask(element_size);
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let acc = simd_element(dst, lane, element_size);
        let a = simd_element(lhs, lane, element_size);
        let b = simd_element(rhs, lane, element_size);
        out |= (f(acc, a, b) & element_mask) << (lane * bits);
    }
    out
}

pub(in crate::arm64::execute) fn simd_fp_elementwise_binary<F32, F64>(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    op32: F32,
    op64: F64,
) -> u128
where
    F32: Fn(f32, f32) -> f32,
    F64: Fn(f64, f64) -> f64,
{
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    match element_size {
        4 => {
            for lane in 0..lanes {
                let a = f32::from_bits(simd_element(lhs, lane, element_size) as u32);
                let b = f32::from_bits(simd_element(rhs, lane, element_size) as u32);
                out |= (op32(a, b).to_bits() as u128) << (lane * 32);
            }
        }
        8 => {
            for lane in 0..lanes {
                let a = f64::from_bits(simd_element(lhs, lane, element_size) as u64);
                let b = f64::from_bits(simd_element(rhs, lane, element_size) as u64);
                out |= (op64(a, b).to_bits() as u128) << (lane * 64);
            }
        }
        _ => {}
    }
    out & simd_vector_mask(vector_size)
}
