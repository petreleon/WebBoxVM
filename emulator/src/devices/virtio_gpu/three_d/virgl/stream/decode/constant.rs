const CMD_SET_CONSTANT_BUFFER: u8 = 12;
const PIPE_SHADER_VERTEX: u32 = 0;
const PIPE_SHADER_FRAGMENT: u32 = 1;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    SetVertex(Option<[u32; 16]>),
    SetFragment(Option<[u32; 4]>),
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_SET_CONSTANT_BUFFER, 0, [PIPE_SHADER_VERTEX, 0]) => Some(Command::SetVertex(None)),
        (CMD_SET_CONSTANT_BUFFER, 0, [PIPE_SHADER_VERTEX, 0, values @ ..]) => {
            let values: [u32; 16] = values.try_into().ok()?;
            values.map(f32::from_bits).iter().all(|value| value.is_finite())
                .then_some(Command::SetVertex(Some(values)))
        }
        (CMD_SET_CONSTANT_BUFFER, 0, [PIPE_SHADER_FRAGMENT, 0]) => Some(Command::SetFragment(None)),
        (CMD_SET_CONSTANT_BUFFER, 0, [PIPE_SHADER_FRAGMENT, 0, red, green, blue, alpha]) => {
            let values = [*red, *green, *blue, *alpha];
            values
                .map(f32::from_bits)
                .iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
                .then_some(Command::SetFragment(Some(values)))
        }
        _ => None,
    }
}
