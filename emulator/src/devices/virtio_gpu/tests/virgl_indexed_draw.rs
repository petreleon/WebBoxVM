use super::super::protocol::*;
use super::{header, virgl_draw_fixture::*, virgl_source_over_state, virgl_viewport_scissor_state};

const INDEX: u32 = 7;

#[test]
fn standard_indexed_draw_vbo_resolves_bounded_u16_and_u32_triangles() {
    let (mut gpu, mut mem) = prepared_nonresident();
    attach_index_buffer(&mut gpu, &mut mem);
    configure_draw(&mut gpu, &mut mem);
    upload_vertices(&mut gpu);
    for (size, offset, data) in [
        (2, 2, &[0xa5, 0x5a, 2, 0, 1, 0, 0, 0][..]),
        (
            4,
            4,
            &[0xa5, 0x5a, 0x55, 0xaa, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0][..],
        ),
    ] {
        gpu.resources
            .get_mut(&INDEX)
            .expect("index resource")
            .pixels[..data.len()]
            .copy_from_slice(data);
        assert_response(
            &mut gpu,
            &mut mem,
            &submit(&index_buffer(INDEX, size, offset)),
            RESP_OK_NODATA,
        );
        let mut command = clear([0.1, 0.2, 0.3, 1.0]);
        command.extend(indexed_draw(0));
        assert_response(&mut gpu, &mut mem, &submit(&command), RESP_OK_NODATA);
        let packet = gpu.take_3d_update();
        assert_eq!(&packet[..4], b"VGD1");
        assert_eq!(&packet[56..72], &gpu.resources[&BUFFER].pixels[32..48]);
        assert_eq!(&packet[72..88], &gpu.resources[&BUFFER].pixels[16..32]);
        assert_eq!(&packet[88..104], &gpu.resources[&BUFFER].pixels[..16]);
        let effect = gpu.pending_3d.last().and_then(|draw| draw.effect.clone());
        assert!(gpu.apply_3d_effect(effect.expect("indexed draw effect")));
    }
    let mut invalid = clear([0.1, 0.2, 0.3, 1.0]);
    invalid.extend(indexed_draw(1));
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&invalid),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(binding(&gpu), Some((4, 4, INDEX)));
    detach(&mut gpu, &mut mem, INDEX);
    assert_eq!(binding(&gpu), None);
}

#[test]
fn index_buffer_binding_rejects_bad_shapes_transactionally() {
    let (mut gpu, mut mem) = prepared();
    attach_index_buffer(&mut gpu, &mut mem);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&index_buffer(INDEX, 2, 2)),
        RESP_OK_NODATA,
    );
    for (words, expected) in [
        (index_buffer(INDEX, 1, 0), RESP_ERR_INVALID_PARAMETER),
        (index_buffer(INDEX, 2, 1), RESP_ERR_INVALID_PARAMETER),
        (index_buffer(BUFFER, 2, 0), RESP_ERR_INVALID_PARAMETER),
        (index_buffer(99, 2, 0), RESP_ERR_INVALID_RESOURCE_ID),
        (vec![word(11, 0, 2), INDEX, 2], RESP_ERR_INVALID_PARAMETER),
    ] {
        assert_response(&mut gpu, &mut mem, &submit(&words), expected);
    }
    assert_eq!(binding(&gpu), Some((2, 2, INDEX)));
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&vec![word(11, 0, 1), 0]),
        RESP_OK_NODATA,
    );
    assert_eq!(binding(&gpu), None);
}

fn configure_draw(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
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

fn index_buffer(resource: u32, index_size: u32, offset: u32) -> Vec<u32> {
    vec![word(11, 0, 3), resource, index_size, offset]
}

fn indexed_draw(start: u32) -> Vec<u32> {
    vec![word(8, 0, 12), start, 3, 4, 1, 1, 0, 0, 0, 0, 0, 2, 0]
}

fn binding(gpu: &super::super::VirtioGpu) -> Option<(u32, u32, u32)> {
    gpu.virgl_contexts[&7]
        .index_buffer()
        .map(|binding| (binding.index_size, binding.offset, binding.resource))
}

fn detach(
    gpu: &mut super::super::VirtioGpu,
    mem: &mut crate::memory::PhysicalMemory,
    resource: u32,
) {
    let mut command = header(CMD_CTX_DETACH_RESOURCE);
    for value in [resource, 0] {
        push_u32(&mut command, value);
    }
    assert_response(gpu, mem, &command, RESP_OK_NODATA);
}
