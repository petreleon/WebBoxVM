use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const COLOR: u32 = 8;
const POSITION: u32 = 7;
const COLOR_FRAG: &str =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL OUT[0], COLOR[0]\nMOV OUT[0], IN[0]\nEND\n";

#[test]
fn standard_split_vertex_buffers_normalize_generic_color_input() {
    let (mut gpu, mut mem) = prepared();
    for resource in [POSITION, COLOR] {
        create_buffer(&mut gpu, &mut mem, resource);
        attach(&mut gpu, &mut mem, resource);
    }
    assert_response(&mut gpu, &mut mem, &submit(&state(true)), RESP_OK_NODATA);
    upload(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(7));
    assert_eq!(packet.len(), 192);
    assert_eq!(
        &packet[56..88],
        floats(&[0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]),
    );
    gpu.resources.get_mut(&POSITION).unwrap().pixels.fill(0);
    gpu.resources.get_mut(&COLOR).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("split draw effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[64, 64, 127, 255]);
}

#[test]
fn split_vertex_layout_requires_every_declared_source_slot() {
    let (mut gpu, mut mem) = prepared();
    for resource in [POSITION, COLOR] {
        create_buffer(&mut gpu, &mut mem, resource);
    }
    attach(&mut gpu, &mut mem, POSITION);
    assert_response(&mut gpu, &mut mem, &submit(&state(false)), RESP_OK_NODATA);
    upload(&mut gpu);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_ERR_INVALID_PARAMETER);
    assert!(gpu.take_3d_update().is_empty());
}

fn state(with_color: bool) -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, TEXTURED_VERT));
    let mut fragment = shader_create(12, 1, COLOR_FRAG);
    fragment[4] = 11;
    state.extend(fragment);
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend([word(1, 5, 9), 10, 0, 0, 0, 31, 0, 0, 1, 31]);
    state.extend([word(2, 5, 1), 10]);
    state.extend(if with_color {
        vec![word(6, 0, 6), 16, 0, POSITION, 16, 0, COLOR]
    } else {
        vec![word(6, 0, 3), 16, 0, POSITION]
    });
    state
}

fn create_buffer(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory, id: u32) {
    let mut command = super::header(CMD_RESOURCE_CREATE_3D);
    for value in [id, 0, 31, 1 << 4, 48, 1, 1, 1, 0, 0, 0, 0] {
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

fn upload(gpu: &mut super::super::VirtioGpu) {
    let positions = [0.0, 0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0];
    let colors = [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    for (resource, values) in [(POSITION, &positions[..]), (COLOR, &colors[..])] {
        gpu.resources.get_mut(&resource).unwrap().pixels[..48].copy_from_slice(&floats(values));
    }
}

fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}
