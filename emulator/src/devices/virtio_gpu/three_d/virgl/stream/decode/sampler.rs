use super::super::super::{VIRGL_OBJECT_SAMPLER_STATE, VIRGL_OBJECT_SAMPLER_VIEW};
use crate::devices::virtio_gpu::resource::FORMAT_B8G8R8A8_UNORM;

const CMD_BIND_SAMPLER_STATES: u8 = 18;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_SAMPLER_VIEWS: u8 = 10;
const FRAGMENT_SHADER: u32 = 1;
const FIXED_SAMPLER_STATE: u32 = 0x1092;
const IDENTITY_SWIZZLE: u32 = 0x688;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    CreateState { handle: u32 },
    DestroyState { handle: u32 },
    BindState { handle: Option<u32> },
    CreateView { handle: u32, resource: u32 },
    DestroyView { handle: u32 },
    BindView { handle: Option<u32> },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_SAMPLER_STATE,
            [handle, FIXED_SAMPLER_STATE, 0, 0, 0, 0, 0, 0, 0],
        ) => Some(Command::CreateState { handle: *handle }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SAMPLER_STATE, [handle]) => {
            Some(Command::DestroyState { handle: *handle })
        }
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_SAMPLER_VIEW,
            [
                handle,
                resource,
                FORMAT_B8G8R8A8_UNORM,
                0,
                0,
                IDENTITY_SWIZZLE,
            ],
        ) => Some(Command::CreateView {
            handle: *handle,
            resource: *resource,
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SAMPLER_VIEW, [handle]) => {
            Some(Command::DestroyView { handle: *handle })
        }
        (CMD_SET_SAMPLER_VIEWS, 0, [FRAGMENT_SHADER, 0, handle]) => Some(Command::BindView {
            handle: (*handle != 0).then_some(*handle),
        }),
        (CMD_BIND_SAMPLER_STATES, 0, [FRAGMENT_SHADER, 0, handle]) => Some(Command::BindState {
            handle: (*handle != 0).then_some(*handle),
        }),
        _ => None,
    }
}
