use super::super::protocol::*;
use super::{virgl_source_over_state, virgl_viewport_scissor_state};

pub(super) use super::virgl_draw_fixture::*;

#[test]
fn standard_draw_vbo_queues_a_webgpu_triangle_after_clear() {
    let (mut gpu, mut mem) = prepared();
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA);
    upload_vertices(&mut gpu);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&draw()),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());

    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGD1");
    assert_eq!(packet.len(), 144);
    assert_eq!(read_u32(&packet, 4), Some(2));
    assert_eq!(read_u32(&packet, 8), Some(1));
    assert_eq!(read_u32(&packet, 20), Some(3));
    assert_eq!(read_u32(&packet, 52), Some(0.25f32.to_bits()));
    assert_eq!(read_u32(&packet, 104), Some(256.0f32.to_bits()));
    assert_eq!(
        &packet[128..144],
        &[192, 1, 0, 0, 80, 1, 0, 0, 128, 0, 0, 0, 96, 0, 0, 0]
    );
    let effect = gpu.pending_3d[0].effect.clone().expect("draw ack effect");
    assert!(gpu.apply_3d_effect(effect));
    assert_eq!(&gpu.resources[&TARGET].pixels[..4], &[77, 51, 26, 255]);
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[middle..middle + 4],
        &[58, 102, 20, 255]
    );
    let clipped = ((384 * 1024 + 400) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[clipped..clipped + 4],
        &[77, 51, 26, 255]
    );
    assert_eq!(&gpu.take_scanout_update()[..4], b"WBGF");
}
