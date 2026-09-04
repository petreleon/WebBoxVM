use super::super::protocol::*;
use super::{virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const VERT: &str = "VERT\nDCL OUT[0], POSITION\nDCL IN[0], POSITION\nDCL OUT[1], GENERIC[0]\nDCL IN[1], GENERIC[0]\nMOV OUT[0].xyzw, IN[0].xyzw\nMOV OUT[1].xyzw, IN[1].xyzw\nEND\n";
const FRAG: &str = "FRAG\nDCL OUT[0], COLOR\nDCL TEMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nTEX TEMP[0].xyzw, IN[0].xyzw, SAMP[0], 2D\nMOV OUT[0].xyzw, TEMP[0].xyzw\nEND\n";

#[test]
fn normalized_tgsi_shapes_reach_the_standard_textured_draw_path() {
    let (mut gpu, mut mem) = prepared_nonresident(); let mut state = surface_create(9, TARGET); state.extend(framebuffer(9));
    let mut vertex = shader_create(11, 0, VERT); vertex[4] = 17; let mut fragment = shader_create(12, 1, FRAG); fragment[4] = 25;
    state.extend(vertex); state.extend(fragment); state.extend(shader_bind(11, 0)); state.extend(shader_bind(12, 1));
    state.extend(virgl_source_over_state(13)); state.extend(virgl_viewport_scissor_state(14)); state.extend(textured_vertex_state());
    state.extend([vec![word(1, 7, 9), 17, 0x1092, 0, 0, 0, 0, 0, 0, 0], vec![word(1, 6, 6), 18, TEXTURE, 1, 0, 0, 0x688], vec![word(10, 0, 3), 1, 0, 18], vec![word(18, 0, 3), 1, 0, 17]].concat());
    assert_response(&mut gpu, &mut mem, &submit(&state), RESP_OK_NODATA); upload_textured_vertices(&mut gpu);
    gpu.resources.get_mut(&TEXTURE).unwrap().pixels.copy_from_slice(&[10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255]);
    let mut command = clear([0.1, 0.2, 0.3, 1.0]); command.extend(draw()); assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
    let packet = gpu.take_3d_update(); assert_eq!(&packet[..4], b"VGD1"); assert_eq!(read_u32(&packet, 4), Some(3));
    let effect = gpu.pending_3d[0].effect.clone().expect("normalized shader draw effect"); assert!(gpu.apply_3d_effect(effect));
    let center = ((384 * 1024 + 512) * 4) as usize; assert_eq!(&gpu.resources[&TARGET].pixels[center..center + 4], &[10, 20, 30, 255]);
}
