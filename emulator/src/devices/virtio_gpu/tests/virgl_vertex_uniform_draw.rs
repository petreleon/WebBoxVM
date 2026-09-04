use super::super::protocol::*;
use super::{
    header, response_type, virgl_draw_fixture::*, virgl_source_over_state,
    virgl_viewport_scissor_state,
};

const COLOR: [f32; 4] = [0.8, 0.4, 0.2, 0.5];
const OFFSET: [f32; 4] = [-0.015625, 0.0, 0.0, 0.0];
const UNIFORM: u32 = 7;
const OFFSET_VERT: &str =
    "VERT\nDCL IN[0]\nDCL CONST[0][0]\nDCL OUT[0], POSITION\nADD OUT[0], IN[0], CONST[0][0]\nEND\n";
const CONSTANT_FRAG: &str =
    "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn vertex_uniform_offsets_the_snapshot_before_the_schema_two_draw() {
    let (mut gpu, mut mem) = prepared_nonresident();
    attach_uniform(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    upload_vertices(&mut gpu);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&inline(UNIFORM, 0, OFFSET)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&inline(UNIFORM, 16, COLOR)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&uniform(0, UNIFORM, 0)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&uniform(1, UNIFORM, 16)),
        RESP_OK_NODATA,
    );
    let packet = render(&mut gpu, &mut mem).expect("vertex-uniform draw");
    assert_eq!(read_u32(&packet, 56), Some(OFFSET[0].to_bits()));
    gpu.resources.get_mut(&UNIFORM).unwrap().pixels.fill(0);
    let effect = gpu.pending_3d[0]
        .effect
        .clone()
        .expect("vertex-uniform effect");
    assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[center..center + 4],
        &[64, 77, 115, 255]
    );
}

#[test]
fn vertex_uniform_requires_a_planar_binding_and_can_be_cleared() {
    let (mut gpu, mut mem) = prepared();
    attach_uniform(&mut gpu, &mut mem);
    configure(&mut gpu, &mut mem);
    upload_vertices(&mut gpu);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&inline(UNIFORM, 16, COLOR)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&uniform(1, UNIFORM, 16)),
        RESP_OK_NODATA,
    );
    assert!(render(&mut gpu, &mut mem).is_none());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&inline(UNIFORM, 0, [0.0, 0.0, 0.0, 1.0])),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&uniform(0, UNIFORM, 0)),
        RESP_OK_NODATA,
    );
    assert!(render(&mut gpu, &mut mem).is_none());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&inline(UNIFORM, 0, OFFSET)),
        RESP_OK_NODATA,
    );
    assert!(render(&mut gpu, &mut mem).is_some());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&clear_uniform(0)),
        RESP_OK_NODATA,
    );
    assert!(render(&mut gpu, &mut mem).is_none());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&uniform(0, UNIFORM, 0)),
        RESP_OK_NODATA,
    );
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
    state.extend(shader_create(11, 0, OFFSET_VERT));
    state.extend(shader_create(12, 1, CONSTANT_FRAG));
    state.extend(shader_bind(11, 0));
    state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13));
    state.extend(virgl_viewport_scissor_state(14));
    state.extend(vertex_state());
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn inline(resource: u32, offset: u32, values: [f32; 4]) -> Vec<u32> {
    let mut words = vec![word(9, 0, 15), resource, 0, 0, 0, 0, offset, 0, 0, 16, 1, 1];
    words.extend(values.map(f32::to_bits));
    words
}

fn uniform(stage: u32, resource: u32, offset: u32) -> Vec<u32> {
    vec![word(27, 0, 5), stage, 0, offset, 16, resource]
}

fn clear_uniform(stage: u32) -> Vec<u32> {
    vec![word(27, 0, 5), stage, 0, 0, 0, 0]
}

fn detach(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_CTX_DETACH_RESOURCE);
    for value in [resource, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn render(
    gpu: &mut super::super::VirtioGpu,
    mem: &mut crate::memory::PhysicalMemory,
) -> Option<Vec<u8>> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    (response_type(&gpu.execute_command(mem, &submit(&command))) == RESP_OK_NODATA)
        .then(|| gpu.take_3d_update())
}
