use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

#[test]
fn standard_repeat_sampler_wraps_a_one_coordinate_in_cpu_and_browser_work() {
    let (mut gpu, mut mem) = prepared();
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&repeat_textured_state()),
        RESP_OK_NODATA,
    );
    upload_textured_vertices(&mut gpu);
    {
        let pixels = &mut gpu.resources.get_mut(&BUFFER).unwrap().pixels;
        for offset in [16, 40, 64] {
            pixels[offset..offset + 4].copy_from_slice(&1.0f32.to_le_bytes());
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
    assert_eq!(read_u32(&packet, 168), Some(0x1080));
    assert_eq!(read_u32(&packet, 172), Some(2));
    assert_eq!(&packet[180..184], &[10, 20, 30, 255]);
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[center..center + 4],
        &[10, 20, 30, 255]
    );
}

fn repeat_textured_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend([word(1, 7, 9), 17, 0x1080, 0, 0, 0, 0, 0, 0, 0]);
    state.extend([word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688]);
    state.extend([word(10, 0, 3), 1, 0, 18]);
    state.extend([word(18, 0, 3), 1, 0, 17]);
    state
}
