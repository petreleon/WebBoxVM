use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

#[test]
fn standard_sampler_view_draws_from_a_queued_texture_snapshot() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend(sampler_state(17));
    state.extend(sampler_view(18));
    state.extend(set_sampler_view(18));
    state.extend(bind_sampler_state(17));
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    gpu.resources
        .get_mut(&TEXTURE)
        .unwrap()
        .pixels
        .copy_from_slice(&[
            10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
        ]);

    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGD1");
    assert_eq!(read_u32(&packet, 4), Some(3));
    assert_eq!(packet.len(), 192);
    assert_eq!(read_u32(&packet, 168), Some(2));
    assert_eq!(read_u32(&packet, 172), Some(2));
    assert_eq!(
        &packet[176..],
        &[
            10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255
        ]
    );
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[center..center + 4],
        &[10, 20, 30, 255]
    );
}

#[test]
fn textured_shader_requires_both_standard_fragment_sampler_bindings() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&command),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());
}

fn sampler_state(handle: u32) -> Vec<u32> {
    vec![word(1, 7, 9), handle, 0x1092, 0, 0, 0, 0, 0, 0, 0]
}

fn sampler_view(handle: u32) -> Vec<u32> {
    vec![word(1, 6, 6), handle, TEXTURE, 1, 0, 0, 0x688]
}

fn set_sampler_view(handle: u32) -> Vec<u32> {
    vec![word(10, 0, 3), 1, 0, handle]
}

fn bind_sampler_state(handle: u32) -> Vec<u32> {
    vec![word(18, 0, 3), 1, 0, handle]
}
