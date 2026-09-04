use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

#[test]
fn standard_linear_sampler_interpolates_two_texels_in_cpu_and_browser_work() {
    let (mut gpu, mut mem) = prepared_nonresident();
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&linear_textured_state()),
        RESP_OK_NODATA,
    );
    upload_textured_vertices(&mut gpu);
    {
        let pixels = &mut gpu.resources.get_mut(&BUFFER).unwrap().pixels;
        for offset in [16, 40, 64] {
            pixels[offset..offset + 4].copy_from_slice(&0.5f32.to_le_bytes());
        }
    }
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
    assert_eq!(read_u32(&packet, 4), Some(5));
    assert_eq!(packet.len(), 196);
    assert_eq!(read_u32(&packet, 168), Some(0x3292));
    assert_eq!(read_u32(&packet, 172), Some(2));
    assert_eq!(&packet[180..184], &[10, 20, 30, 255]);
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[center..center + 4],
        &[25, 35, 45, 255]
    );
}

#[test]
fn linear_sampler_state_follows_a_completed_repeat_draw_with_fresh_handles() {
    let (mut gpu, mut mem) = prepared_nonresident();
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&textured_state(0, 0x1080, true)),
        RESP_OK_NODATA,
    );
    upload_textured_vertices(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    assert_eq!(read_u32(&gpu.take_3d_update(), 168), Some(0x1080));
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&textured_state(32, 0x3292, false)),
        RESP_OK_NODATA,
    );
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 168), Some(0x3292));
}

fn linear_textured_state() -> Vec<u32> {
    textured_state(0, 0x3292, true)
}

fn textured_state(base: u32, sampler: u32, create_surface: bool) -> Vec<u32> {
    let mut state = create_surface
        .then(|| surface_create(1, TARGET))
        .unwrap_or_default();
    state.extend(framebuffer(1));
    state.extend(shader_create(base + 21, 0, TEXTURED_VERT));
    state.extend(shader_create(base + 22, 1, TEXTURED_FRAG));
    state.extend(shader_bind(base + 21, 0));
    state.extend(shader_bind(base + 22, 1));
    state.extend(virgl_source_over_state(base + 23));
    state.extend(virgl_viewport_scissor_state(base + 24));
    state.extend(textured_vertex_state_with_handle(base + 20));
    state.extend([word(1, 7, 9), base + 17, sampler, 0, 0, 0, 0, 0, 0, 0]);
    state.extend([word(1, 6, 6), base + 18, TEXTURE, 1, 0, 0, 0x688]);
    state.extend([word(10, 0, 3), 1, 0, base + 18]);
    state.extend([word(18, 0, 3), 1, 0, base + 17]);
    state
}

fn textured_vertex_state_with_handle(handle: u32) -> Vec<u32> {
    let mut state = textured_vertex_state();
    state[1] = handle;
    state[11] = handle;
    state
}
