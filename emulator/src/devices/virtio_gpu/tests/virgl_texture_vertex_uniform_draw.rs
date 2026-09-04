use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const UNIFORM: u32 = 7;
const OFFSET: [f32; 4] = [0.015625, 0.0, 0.0, 0.0];
const VERT: &str = "VERT\nDCL OUT[1], GENERIC[0]\nDCL CONST[0][0]\nDCL IN[0], POSITION\nDCL OUT[0], POSITION\nDCL IN[1], GENERIC[0]\nADD OUT[0].xyzw, CONST[0][0].xyzw, IN[0].xyzw\nMOV OUT[1].xyzw, IN[1].xyzw\nEND\n";
const FRAG: &str = "FRAG\nDCL CONST[0][0]\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMUL OUT[0], TEMP[0], CONST[0][0]\nEND\n";

#[test]
fn vertex_uniform_offset_and_fragment_constant_share_the_texture_color_snapshot() {
    let (mut gpu, mut mem) = prepared();
    attach_uniform(&mut gpu, &mut mem);
    assert_response(&mut gpu, &mut mem, &submit(&state()), RESP_OK_NODATA);
    upload_textured_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.chunks_exact_mut(4)
        .for_each(|pixel| pixel.copy_from_slice(&[128, 128, 128, 255]));
    store(&mut gpu, OFFSET);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(vertex_uniform());
    command.extend(fragment_constant([0.5, 0.5, 0.5, 1.0]));
    command.extend(draw());
    assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update();
    assert_eq!(read_u32(&packet, 4), Some(8));
    assert_eq!(packet.len(), 244);
    assert_eq!(read_u32(&packet, 56), Some(OFFSET[0].to_bits()));
    assert_eq!(read_u32(&packet, 72), Some(0.5f32.to_bits()));
    gpu.resources.get_mut(&BUFFER).unwrap().pixels.fill(0);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.fill(0);
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0].effect.clone().expect("texture vertex-uniform effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[64, 64, 64, 255]);
}

fn state() -> Vec<u32> {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    let mut vertex = shader_create(11, 0, VERT); vertex[4] = 20;
    let mut fragment = shader_create(12, 1, FRAG); fragment[4] = 30;
    state.extend(vertex); state.extend(fragment);
    state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14));
    state.extend(textured_vertex_state());
    state.extend([vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0], vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688], vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17]].concat());
    state
}

fn attach_uniform(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [UNIFORM, 0, 64, 1 << 6, 32, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut create, value);
    }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [UNIFORM, 0] { push_u32(&mut attach, value); }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn store(gpu: &mut super::super::VirtioGpu, values: [f32; 4]) {
    let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels[..16].copy_from_slice(&bytes);
}

fn vertex_uniform() -> Vec<u32> {
    vec![word(27, 0, 5), 0, 0, 0, 16, UNIFORM]
}

fn fragment_constant(color: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(12, 0, 6), 1, 0];
    words.extend(color.map(f32::to_bits));
    words
}
