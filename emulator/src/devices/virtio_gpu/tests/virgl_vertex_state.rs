use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::memory::PhysicalMemory;

const BUFFER_ID: u32 = 1;
const BUFFER_BYTES: u32 = 12;
const CONTEXT_ID: u32 = 7;

#[test]
fn standard_vertex_input_state_binds_and_releases_a_byte_buffer() {
    let (mut gpu, mut mem) = prepared_buffer();
    let mut words = vertex_elements_create(9, 64);
    words.extend(vertex_elements_bind(9));
    words.extend(vertex_buffer(Some((1, 4, BUFFER_ID))));
    assert_response(&mut gpu, &mut mem, &submit(&words), RESP_OK_NODATA);
    assert_eq!(binding(&gpu), Some((1, 4, BUFFER_ID)));
    assert_eq!(element(&gpu), Some((0, 0, 0, 64)));
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&vertex_buffer(None)),
        RESP_OK_NODATA,
    );
    assert_eq!(binding(&gpu), None);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&vertex_buffer(Some((1, 4, BUFFER_ID)))),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &context_resource(CMD_CTX_DETACH_RESOURCE, BUFFER_ID),
        RESP_OK_NODATA,
    );
    assert_eq!(binding(&gpu), None);
    assert_eq!(element(&gpu), Some((0, 0, 0, 64)));
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&vertex_elements_destroy(9)),
        RESP_OK_NODATA,
    );
    assert_eq!(element(&gpu), None);
}

#[test]
fn vertex_input_rejects_bad_or_partial_streams_without_mutating_state() {
    let (mut gpu, mut mem) = prepared_buffer();
    let mut initial = vertex_elements_create(9, 64);
    initial.extend(vertex_elements_bind(9));
    initial.extend(vertex_buffer(Some((1, 4, BUFFER_ID))));
    assert_response(&mut gpu, &mut mem, &submit(&initial), RESP_OK_NODATA);
    for (words, expected) in [
        (
            combined(
                vertex_buffer(Some((1, 0, BUFFER_ID))),
                vertex_elements_create(10, 1),
            ),
            RESP_ERR_INVALID_PARAMETER,
        ),
        (
            vertex_buffer(Some((1, BUFFER_BYTES, BUFFER_ID))),
            RESP_ERR_INVALID_PARAMETER,
        ),
        (vertex_buffer(Some((1, 0, 2))), RESP_ERR_INVALID_RESOURCE_ID),
        (vertex_buffer(Some((0, 1, 0))), RESP_ERR_INVALID_PARAMETER),
        (vertex_elements_create(0, 64), RESP_ERR_INVALID_PARAMETER),
        (
            vec![command_header(6, 0, 6), 1, 0, BUFFER_ID, 1, 0, BUFFER_ID],
            RESP_ERR_INVALID_PARAMETER,
        ),
    ] {
        assert_response(&mut gpu, &mut mem, &submit(&words), expected);
    }
    assert_eq!(binding(&gpu), Some((1, 4, BUFFER_ID)));
    assert_eq!(element(&gpu), Some((0, 0, 0, 64)));
}

fn prepared_buffer() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &buffer_create(), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &context_resource(CMD_CTX_ATTACH_RESOURCE, BUFFER_ID),
        RESP_OK_NODATA,
    );
    (gpu, mem)
}

fn buffer_create() -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [BUFFER_ID, 0, 64, 1 << 4, BUFFER_BYTES, 1, 1, 1, 0, 0, 0, 0] {
        push_u32(&mut command, value);
    }
    command
}

fn virgl_context() -> Vec<u8> {
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 5);
    push_u32(&mut command, VIRGL_CAPSET_ID);
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    command
}

fn context_resource(kind: u32, resource: u32) -> Vec<u8> {
    let mut command = header(kind);
    push_u32(&mut command, resource);
    push_u32(&mut command, 0);
    command
}

fn submit(words: &[u32]) -> Vec<u8> {
    let mut command = header(CMD_SUBMIT_3D);
    push_u32(&mut command, (words.len() * 4) as u32);
    push_u32(&mut command, 0);
    for word in words {
        push_u32(&mut command, *word);
    }
    command
}

fn vertex_elements_create(handle: u32, format: u32) -> Vec<u32> {
    vec![command_header(1, 5, 5), handle, 0, 0, 0, format]
}

fn vertex_elements_bind(handle: u32) -> Vec<u32> {
    vec![command_header(2, 5, 1), handle]
}

fn vertex_elements_destroy(handle: u32) -> Vec<u32> {
    vec![command_header(3, 5, 1), handle]
}

fn vertex_buffer(binding: Option<(u32, u32, u32)>) -> Vec<u32> {
    binding.map_or_else(
        || vec![command_header(6, 0, 0)],
        |(stride, offset, resource)| vec![command_header(6, 0, 3), stride, offset, resource],
    )
}

fn combined(mut first: Vec<u32>, second: Vec<u32>) -> Vec<u32> {
    first.extend(second);
    first
}

fn binding(gpu: &VirtioGpu) -> Option<(u32, u32, u32)> {
    gpu.virgl_contexts[&CONTEXT_ID]
        .vertex_buffer()
        .map(|binding| (binding.stride, binding.offset, binding.resource))
}

fn element(gpu: &VirtioGpu) -> Option<(u32, u32, u32, u32)> {
    gpu.virgl_contexts[&CONTEXT_ID]
        .bound_vertex_element()
        .map(|element| {
            (
                element.offset,
                element.divisor,
                element.buffer_index,
                element.format,
            )
        })
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
