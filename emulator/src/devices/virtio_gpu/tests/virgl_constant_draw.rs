use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const COLOR: [f32; 4] = [0.8, 0.4, 0.2, 0.5];
const CONSTANT_FRAG: &str =
    "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";

#[test]
fn inline_fragment_constants_render_through_the_solid_packet_route() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&constants(COLOR)),
        RESP_OK_NODATA,
    );
    let packet = render(&mut gpu, &mut mem);
    assert_eq!(
        [40, 44, 48, 52].map(|offset| read_u32(&packet, offset)),
        COLOR.map(f32::to_bits).map(Some),
    );
    let effect = gpu.pending_3d[0]
        .effect
        .clone()
        .expect("constant draw effect");
    assert!(gpu.apply_3d_effect(effect));
    let middle = ((384 * 1024 + 512) * 4) as usize;
    assert_eq!(
        &gpu.resources[&TARGET].pixels[middle..middle + 4],
        &[64, 77, 115, 255]
    );
}

#[test]
fn invalid_inline_constant_updates_preserve_the_last_valid_binding() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&constants(COLOR)),
        RESP_OK_NODATA,
    );
    let mut wrong_slot = constants(COLOR);
    wrong_slot[2] = 1;
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&wrong_slot),
        RESP_ERR_INVALID_PARAMETER,
    );
    let mut wrong_stage = constants(COLOR);
    wrong_stage[1] = 0;
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&wrong_stage),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&constants([f32::NAN, 0.4, 0.2, 0.5])),
        RESP_ERR_INVALID_PARAMETER,
    );
    let packet = render(&mut gpu, &mut mem);
    assert_eq!(read_u32(&packet, 40), Some(COLOR[0].to_bits()));
}

#[test]
fn clearing_inline_constants_rejects_the_constant_fragment_draw() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&constants(COLOR)),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&clear_constants()),
        RESP_OK_NODATA,
    );
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&command),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert!(gpu.take_3d_update().is_empty());
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

fn constants(color: [f32; 4]) -> Vec<u32> {
    let mut command = vec![word(12, 0, 6), 1, 0];
    command.extend(color.map(f32::to_bits));
    command
}

fn clear_constants() -> Vec<u32> {
    vec![word(12, 0, 2), 1, 0]
}

fn render(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) -> Vec<u8> {
    let mut command = clear([0.1, 0.2, 0.3, 1.0]);
    command.extend(draw());
    assert_response(gpu, mem, &submit(&command), RESP_OK_NODATA);
    gpu.take_3d_update()
}
