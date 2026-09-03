use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn color(bits: [u32; 4]) -> Result<[f32; 4], u32> {
    let color = bits.map(f32::from_bits);
    color
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .then_some(color)
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}
