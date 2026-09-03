use super::super::super::MAX_VIRGL_FRAGMENT_SAMPLERS;
use super::super::super::{
    SamplerConfig, SamplerState, VIRGL_OBJECT_SAMPLER_STATE, VIRGL_OBJECT_SAMPLER_VIEW,
};
use crate::devices::virtio_gpu::resource::sampled_texture_format;

const CMD_BIND_SAMPLER_STATES: u8 = 18;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_SAMPLER_VIEWS: u8 = 10;
const FRAGMENT_SHADER: u32 = 1;
const IDENTITY_SWIZZLE: u32 = 0x688;

#[derive(Clone)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    CreateState {
        handle: u32,
        state: SamplerState,
    },
    DestroyState {
        handle: u32,
    },
    BindState {
        start: usize,
        handles: Vec<Option<u32>>,
    },
    CreateView {
        handle: u32,
        resource: u32,
        format: u32,
    },
    DestroyView {
        handle: u32,
    },
    BindView {
        start: usize,
        handles: Vec<Option<u32>>,
    },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_SAMPLER_STATE, [handle, state, 0, 0, 0, 0, 0, 0, 0]) => {
            sampler_state(*state).map(|state| Command::CreateState {
                handle: *handle,
                state,
            })
        }
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SAMPLER_STATE, [handle]) => {
            Some(Command::DestroyState { handle: *handle })
        }
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_SAMPLER_VIEW,
            [handle, resource, format, 0, 0, IDENTITY_SWIZZLE],
        ) if sampled_texture_format(*format) => Some(Command::CreateView {
            handle: *handle,
            resource: *resource,
            format: *format,
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SAMPLER_VIEW, [handle]) => {
            Some(Command::DestroyView { handle: *handle })
        }
        (CMD_SET_SAMPLER_VIEWS, 0, [FRAGMENT_SHADER, start, handles @ ..]) => {
            bindings(*start, handles).map(|(start, handles)| Command::BindView { start, handles })
        }
        (CMD_BIND_SAMPLER_STATES, 0, [FRAGMENT_SHADER, start, handles @ ..]) => {
            bindings(*start, handles).map(|(start, handles)| Command::BindState { start, handles })
        }
        _ => None,
    }
}

fn sampler_state(word: u32) -> Option<SamplerState> {
    Some(SamplerState {
        config: SamplerConfig::from_wire(word)?,
    })
}

fn bindings(start: u32, handles: &[u32]) -> Option<(usize, Vec<Option<u32>>)> {
    let start = usize::try_from(start).ok()?;
    let end = start.checked_add(handles.len())?;
    (!handles.is_empty() && end <= MAX_VIRGL_FRAGMENT_SAMPLERS).then(|| {
        (
            start,
            handles
                .iter()
                .map(|handle| (*handle != 0).then_some(*handle))
                .collect(),
        )
    })
}
