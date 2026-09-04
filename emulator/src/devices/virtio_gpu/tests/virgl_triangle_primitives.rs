use super::super::protocol::*;
use super::{
    header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state,
};

const INDEX: u32 = 7;

#[test]
fn standard_triangle_strip_normalizes_to_a_bounded_triangle_list() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let bytes = upload(&mut gpu);
    let mut strip = draw();
    strip[2..4].copy_from_slice(&[4, 5]);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(strip);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20].map(|offset| read_u32(&packet, offset)), [Some(2), Some(6)]);
    assert_eq!(packet.len(), 192);
    assert_eq!(&packet[56..104], &bytes[..48]);
    assert_eq!(&packet[104..152], [&bytes[32..48], &bytes[16..32], &bytes[48..64]].concat());
    assert_drawn(&mut gpu);
}

#[test]
fn standard_triangle_fan_retains_its_first_spoke() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let bytes = upload(&mut gpu);
    let mut fan = draw();
    fan[2..4].copy_from_slice(&[4, 6]);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(fan);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 20].map(|offset| read_u32(&packet, offset)), [Some(2), Some(6)]);
    assert_eq!(&packet[56..104], &bytes[..48]);
    assert_eq!(&packet[104..152], [&bytes[..16], &bytes[32..48], &bytes[48..64]].concat());
    assert_drawn(&mut gpu);
}

#[test]
fn indexed_triangle_strip_expands_resolved_index_values() {
    let (mut gpu, mut mem) = prepared();
    attach_index_buffer(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    let bytes = upload(&mut gpu);
    gpu.resources
        .get_mut(&INDEX)
        .expect("index buffer")
        .pixels[..8]
        .copy_from_slice(&[2, 0, 1, 0, 0, 0, 3, 0]);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&vec![word(11, 0, 3), INDEX, 2, 0]),
        RESP_OK_NODATA,
    );
    let mut strip = draw();
    strip[2..5].copy_from_slice(&[4, 5, 1]);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(strip);
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[56..72], &bytes[32..48]);
    assert_eq!(&packet[72..88], &bytes[16..32]);
    assert_eq!(&packet[88..104], &bytes[..16]);
    assert_eq!(&packet[104..152], [&bytes[..16], &bytes[16..32], &bytes[48..64]].concat());
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn upload(gpu: &mut super::super::VirtioGpu) -> Vec<u8> {
    let positions = [
        -0.75, 0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0, 0.75, 0.75, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = positions.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources
        .get_mut(&BUFFER)
        .expect("vertex buffer")
        .pixels[..bytes.len()]
        .copy_from_slice(&bytes);
    bytes
}

fn assert_drawn(gpu: &mut super::super::VirtioGpu) {
    let effect = gpu.pending_3d[0].effect.clone().expect("triangle draw effect");
    assert!(gpu.apply_3d_effect(effect));
    assert!(gpu.resources[&TARGET]
        .pixels
        .chunks_exact(4)
        .any(|pixel| pixel == [58, 102, 20, 255]));
}

fn attach_index_buffer(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [INDEX, 0, 64, 1 << 5, 16, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut create, value);
    }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [INDEX, 0] {
        push_u32(&mut attach, value);
    }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}
