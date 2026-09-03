use super::super::VirtioGpu;
use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};
use crate::memory::PhysicalMemory;

const RIGHT_TEXTURE: u32 = 7;

#[test]
fn standard_fragment_two_sampler_slots_multiply_bounded_texture_snapshots() {
    let (mut gpu, mut mem) = prepared();
    attach_right_texture(&mut gpu, &mut mem);
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_MULTIPLY_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend(sampler_state(17));
    state.extend(sampler_state(19));
    state.extend(sampler_view(18, TEXTURE));
    state.extend(sampler_view(20, RIGHT_TEXTURE));
    state.extend(set_sampler_views([18, 20]));
    state.extend(bind_sampler_states([17, 19]));
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    fill_texture(&mut gpu, TEXTURE, [100, 100, 100, 255]);
    fill_texture(&mut gpu, RIGHT_TEXTURE, [128, 128, 128, 255]);

    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(4));
    assert_eq!(packet.len(), 216);
    assert_eq!(
        [168, 172, 176, 180].map(|offset| read_u32(&packet, offset)),
        [Some(2); 4]
    );
    assert!(
        packet[184..200]
            .chunks_exact(4)
            .all(|pixel| pixel == [100, 100, 100, 255])
    );
    assert!(
        packet[200..]
            .chunks_exact(4)
            .all(|pixel| pixel == [128, 128, 128, 255])
    );
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[center..center + 4],
        &[50, 50, 50, 255]
    );
}

fn attach_right_texture(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [RIGHT_TEXTURE, 2, 1, 1 << 3, 2, 2, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut create, value);
    }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [RIGHT_TEXTURE, 0] {
        push_u32(&mut attach, value);
    }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn fill_texture(gpu: &mut VirtioGpu, resource: u32, pixel: [u8; 4]) {
    gpu.resources
        .get_mut(&resource)
        .expect("attached test texture")
        .pixels
        .chunks_exact_mut(4)
        .for_each(|destination| destination.copy_from_slice(&pixel));
}

fn sampler_state(handle: u32) -> Vec<u32> {
    vec![word(1, 7, 9), handle, 0x1092, 0, 0, 0, 0, 0, 0, 0]
}

fn sampler_view(handle: u32, resource: u32) -> Vec<u32> {
    vec![word(1, 6, 6), handle, resource, 1, 0, 0, 0x688]
}

fn set_sampler_views(handles: [u32; 2]) -> Vec<u32> {
    vec![word(10, 0, 4), 1, 0, handles[0], handles[1]]
}

fn bind_sampler_states(handles: [u32; 2]) -> Vec<u32> {
    vec![word(18, 0, 4), 1, 0, handles[0], handles[1]]
}
