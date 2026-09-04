use super::super::protocol::*;
use super::{
    header, response_type, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state,
};

const COLOR: [f32; 4] = [0.8, 0.4, 0.2, 0.5];
const UNIFORM: u32 = 7;
const CONSTANT_FRAG: &str =
    "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn fragment_uniform_buffer_renders_and_snapshots_a_nonzero_offset() {
    let (mut gpu, mut mem) = prepared();
    attach_uniform(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    store(&mut gpu, 4, COLOR);
    assert_response(&mut gpu, &mut mem, &submit(&uniform(UNIFORM, 4)), RESP_OK_NODATA);
    let packet = render(&mut gpu, &mut mem).expect("uniform draw");
    assert_eq!(
        [40, 44, 48, 52].map(|offset| read_u32(&packet, offset)),
        COLOR.map(f32::to_bits).map(Some),
    );
    store(&mut gpu, 4, [0.1, 0.2, 0.3, 1.0]);
    let effect = gpu.pending_3d[0].effect.clone().expect("uniform draw effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[middle..middle + 4],
        &[64, 77, 115, 255]
    );
}

#[test]
fn uniform_binding_rejects_bad_shapes_transactionally_and_can_unbind() {
    let (mut gpu, mut mem) = prepared();
    attach_uniform(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    store(&mut gpu, 4, COLOR);
    assert_response(&mut gpu, &mut mem, &submit(&uniform(UNIFORM, 4)), RESP_OK_NODATA);
    for (words, expected) in [
        (uniform(BUFFER, 4), RESP_ERR_INVALID_PARAMETER),
        (uniform(UNIFORM, 2), RESP_ERR_INVALID_PARAMETER),
        (uniform(UNIFORM, 20), RESP_ERR_INVALID_PARAMETER),
        (vec![word(27, 0, 4), 1, 0, 4, 16], RESP_ERR_INVALID_PARAMETER),
        (vec![word(27, 0, 5), 0, 0, 4, 16, UNIFORM], RESP_ERR_INVALID_PARAMETER),
        (uniform(99, 4), RESP_ERR_INVALID_RESOURCE_ID),
    ] {
        assert_response(&mut gpu, &mut mem, &submit(&words), expected);
    }
    let packet = render(&mut gpu, &mut mem).expect("preserved uniform binding");
    assert_eq!(read_u32(&packet, 40), Some(COLOR[0].to_bits()));
    assert_response(&mut gpu, &mut mem, &submit(&clear_uniform()), RESP_OK_NODATA);
    assert!(render(&mut gpu, &mut mem).is_none());
}

#[test]
fn detaching_a_uniform_resource_clears_the_constant_source() {
    let (mut gpu, mut mem) = prepared();
    attach_uniform(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    store(&mut gpu, 4, COLOR);
    assert_response(&mut gpu, &mut mem, &submit(&uniform(UNIFORM, 4)), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &detach(UNIFORM), RESP_OK_NODATA);
    assert!(render(&mut gpu, &mut mem).is_none());
}

fn attach_uniform(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut create = header(CMD_RESOURCE_CREATE_3D);
    for value in [UNIFORM, 0, 64, 1 << 6, 32, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut create, value);
    }
    assert_response(gpu, mem, &create, RESP_OK_NODATA);
    let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
    for value in [UNIFORM, 0] {
        push_u32(&mut attach, value);
    }
    assert_response(gpu, mem, &attach, RESP_OK_NODATA);
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    state.extend(shader_create(11, 0, VERT));
    state.extend(shader_create(12, 1, CONSTANT_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
    upload_vertices(gpu);
}

fn store(gpu: &mut super::super::VirtioGpu, offset: usize, color: [f32; 4]) {
    let bytes: Vec<u8> = color.into_iter().flat_map(f32::to_le_bytes).collect();
    gpu.resources
        .get_mut(&UNIFORM)
        .expect("uniform resource")
        .pixels[offset..offset + bytes.len()]
        .copy_from_slice(&bytes);
}

fn uniform(resource: u32, offset: u32) -> Vec<u32> {
    vec![word(27, 0, 5), 1, 0, offset, 16, resource]
}

fn clear_uniform() -> Vec<u32> {
    vec![word(27, 0, 5), 1, 0, 0, 0, 0]
}

fn detach(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_CTX_DETACH_RESOURCE);
    for value in [resource, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn render(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) -> Option<Vec<u8>> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    (response_type(&gpu.execute_command(mem, &submit(&command))) == RESP_OK_NODATA)
        .then(|| gpu.take_3d_update())
}
