use super::super::protocol::*;
use super::virgl_draw_fixture::*;

const CONTEXT_ID: u32 = 7;

#[test]
fn standard_fragment_sampler_state_binds_a_sampled_texture_and_clears_on_detach() {
    let (mut gpu, mut mem) = prepared();
    let mut words = sampler_state_create(17);
    words.extend(sampler_view_create(18, TEXTURE));
    words.extend(set_sampler_view(18));
    words.extend(bind_sampler_state(17));
    assert_response(&mut gpu, &mut mem, &submit(&words), RESP_OK_NODATA);
    assert_eq!(
        gpu.virgl_contexts[&CONTEXT_ID].bound_sampled_resource(),
        Some(TEXTURE)
    );
    assert_response(&mut gpu, &mut mem, &detach(TEXTURE), RESP_OK_NODATA);
    assert_eq!(
        gpu.virgl_contexts[&CONTEXT_ID].bound_sampled_resource(),
        None
    );
}

#[test]
fn sampler_state_rejects_wrong_wire_shape_without_mutating_the_binding() {
    let (mut gpu, mut mem) = prepared();
    let mut initial = sampler_state_create(17);
    initial.extend(sampler_view_create(18, TEXTURE));
    initial.extend(set_sampler_view(18));
    initial.extend(bind_sampler_state(17));
    assert_response(&mut gpu, &mut mem, &submit(&initial), RESP_OK_NODATA);
    for words in [
        vec![word(1, 7, 9), 19, 0x1093, 0, 0, 0, 0, 0, 0, 0],
        vec![word(10, 0, 3), 0, 0, 18],
        vec![word(18, 0, 3), 1, 2, 17],
        vec![word(10, 0, 4), 1, 1, 18, 18],
        vec![word(18, 0, 4), 1, 0, 17, 99],
        sampler_view_create(19, TARGET),
    ] {
        assert_response(
            &mut gpu,
            &mut mem,
            &submit(&words),
            RESP_ERR_INVALID_PARAMETER,
        );
    }
    assert_eq!(
        gpu.virgl_contexts[&CONTEXT_ID].bound_sampled_resource(),
        Some(TEXTURE)
    );
}

fn sampler_state_create(handle: u32) -> Vec<u32> {
    vec![word(1, 7, 9), handle, 0x1092, 0, 0, 0, 0, 0, 0, 0]
}

fn sampler_view_create(handle: u32, resource: u32) -> Vec<u32> {
    vec![word(1, 6, 6), handle, resource, 1, 0, 0, 0x688]
}

fn set_sampler_view(handle: u32) -> Vec<u32> {
    vec![word(10, 0, 3), 1, 0, handle]
}

fn bind_sampler_state(handle: u32) -> Vec<u32> {
    vec![word(18, 0, 3), 1, 0, handle]
}

fn detach(resource: u32) -> Vec<u8> {
    let mut command = super::header(CMD_CTX_DETACH_RESOURCE);
    for value in [resource, 0] {
        push_u32(&mut command, value);
    }
    command
}
