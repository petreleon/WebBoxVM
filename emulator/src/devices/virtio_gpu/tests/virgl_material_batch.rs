use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const TEXTURE_BUFFER: u32 = 8;
const TEXTURE_VERT: &str = "VERT\nDCL IN[0..2]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nDCL OUT[2], GENERIC[1]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nMOV OUT[2], IN[2]\nEND\n";
const TEXTURE_FRAG: &str = "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL IN[1], GENERIC[1], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[1], SAMP[0], 2D\nMUL OUT[0], TEMP[0], IN[0]\nEND\n";

#[test]
fn standard_mixed_material_draws_share_one_ordered_batch() {
    let (mut gpu, mut mem) = prepared();
    add_texture_buffer(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &submit(&solid_state()), RESP_OK_NODATA);
    upload_vertices(&mut gpu); upload_texture_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4).for_each(|pixel| pixel.copy_from_slice(&[128, 128, 128, 255]));
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw()); command.extend(texture_state()); command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGM1"); assert_eq!([4, 12, 16, 20, 24, 44].map(|at| read_u32(&packet, at)), [Some(1), Some(1024), Some(768), Some(2), Some(0), Some(0)]);
    assert_eq!([48, 52, 56, 164, 168, 172].map(|at| read_u32(&packet, at)), [Some(1), Some(0), Some(3), Some(5), Some(0), Some(3)]);
    assert_eq!(packet.len(), 364);
    gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels.fill(0); gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("mixed material batch effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[32, 32, 64, 255]);
}

fn add_texture_buffer(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [TEXTURE_BUFFER, 0, 31, 1 << 4, 120, 1, 1, 1, 0, 0, 0, 0] { push_u32(&mut create, value); }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE); for value in [TEXTURE_BUFFER, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn solid_state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT)); state.extend(shader_create(12, 1, FRAG));
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1)); state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14)); state.extend(vertex_state()); state
}

fn texture_state() -> Vec<u32> {
    let mut vertex = shader_create(21, 0, TEXTURE_VERT); vertex[4] = 21;
    let mut fragment = shader_create(22, 1, TEXTURE_FRAG); fragment[4] = 30;
    [
        vertex, fragment, shader_bind(21, 0), shader_bind(22, 1), texture_vertex_state(),
        vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0],
        vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688],
        vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17],
    ].concat()
}

fn texture_vertex_state() -> Vec<u32> {
    [
        vec![word(1, 5, 13), 10, 0, 0, 0, 31, 16, 0, 0, 31, 32, 0, 0, 29],
        vec![word(2, 5, 1), 10], vec![word(6, 0, 3), 40, 0, TEXTURE_BUFFER],
    ].concat()
}

fn upload_texture_vertices(gpu: &mut super::super::VirtioGpu) {
    let vertices = [
        0.0, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        -0.75, -0.75, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        0.75, -0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
    ];
    let bytes: Vec<u8> = vertices.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&TEXTURE_BUFFER).unwrap().pixels[..bytes.len()].copy_from_slice(&bytes);
}
