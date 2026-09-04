use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const POSITION: u32 = 7;
const UV: u32 = 8;

#[test]
fn standard_split_position_and_uv_buffers_snapshot_a_textured_draw() {
    let (mut gpu, mut mem) = prepared();
    create_buffer(&mut gpu, &mut mem, POSITION, 31, 48);
    create_buffer(&mut gpu, &mut mem, UV, 29, 24);
    for resource in [POSITION, UV] {
        attach(&mut gpu, &mut mem, resource);
    }
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA);
    write(&mut gpu, POSITION, &[0.0, 0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0]);
    write(&mut gpu, UV, &[0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.copy_from_slice(&[
        10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
    ]);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(3));
    assert_eq!(packet.len(), 192);
    assert_eq!(&packet[56..80], floats(&[0.0, 0.75, 0.0, 1.0, 0.0, 1.0]));
    let effect = gpu.pending_3d[0].effect.clone().expect("split texture effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[10, 20, 30, 255]);
}

fn state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    state.extend(shader_create(12, 1, TEXTURED_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend([word(1, 5, 9), 10, 0, 0, 0, 31, 0, 0, 1, 29]);
    state.extend([word(2, 5, 1), 10]);
    state.extend([word(6, 0, 6), 16, 0, POSITION, 8, 0, UV]);
    state.extend([word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0]);
    state.extend([word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688]);
    state.extend([word(10, 0, 3), 1, 0, 18]);
    state.extend([word(18, 0, 3), 1, 0, 17]);
    state
}

fn create_buffer(
    gpu: &mut super::super::VirtioGpu,
    mem: &mut crate::memory::PhysicalMemory,
    id: u32,
    format: u32,
    width: u32,
) {
    let mut command = super::header(CMD_RESOURCE_CREATE_3D);
    for value in [id, 0, format, 1 << 4, width, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    assert_response(gpu, mem, &command, RESP_OK_NODATA);
}

fn attach(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory, id: u32) {
    let mut command = super::header(CMD_CTX_ATTACH_RESOURCE);
    for value in [id, 0] {
        push_u32(&mut command, value);
    }
    assert_response(gpu, mem, &command, RESP_OK_NODATA);
}

fn write(gpu: &mut super::super::VirtioGpu, resource: u32, values: &[f32]) {
    let bytes = floats(values);
    gpu.resources.get_mut(&resource).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}

fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}
