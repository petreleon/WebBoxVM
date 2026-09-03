use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::memory::PhysicalMemory;

const BUFFER_BYTES: u32 = 12;

#[test]
fn resource_copy_region_copies_byte_ranges_and_snapshots_self_overlap() {
    let (mut gpu, mut mem) = prepared_buffers();
    gpu.resources
        .get_mut(&1)
        .unwrap()
        .pixels
        .copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&copy(2, 2, 1, 1, 5)),
        RESP_OK_NODATA,
    );
    assert_eq!(&gpu.resources[&2].pixels[2..7], &[1, 2, 3, 4, 5]);

    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&copy(1, 3, 1, 0, 6)),
        RESP_OK_NODATA,
    );
    assert_eq!(
        &gpu.resources[&1].pixels,
        &[0, 1, 2, 0, 1, 2, 3, 4, 5, 9, 10, 11]
    );
}

#[test]
fn resource_copy_region_rejects_invalid_buffer_boxes_without_mutation() {
    let (mut gpu, mut mem) = prepared_buffers();
    gpu.resources
        .get_mut(&1)
        .unwrap()
        .pixels
        .copy_from_slice(&[9; BUFFER_BYTES as usize]);
    let before = gpu.resources[&2].pixels.clone();
    for words in [
        copy(2, 10, 1, 0, 3),
        copy_with_y(2, 0, 1, 0, 1),
        copy_with_height(2, 0, 1, 0, 2),
    ] {
        assert_response(
            &mut gpu,
            &mut mem,
            &submit(&words),
            RESP_ERR_INVALID_PARAMETER,
        );
    }
    assert_eq!(gpu.resources[&2].pixels, before);
}

fn prepared_buffers() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    for resource in [1, 2] {
        assert_response(&mut gpu, &mut mem, &buffer_create(resource), RESP_OK_NODATA);
    }
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    for resource in [1, 2] {
        let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
        push_u32(&mut attach, resource);
        push_u32(&mut attach, 0);
        assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
    }
    (gpu, mem)
}

fn buffer_create(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [resource, 0, 64, 1 << 4, BUFFER_BYTES, 1, 1, 1, 0, 0, 0, 0] {
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

fn submit(words: &[u32]) -> Vec<u8> {
    let mut command = header(CMD_SUBMIT_3D);
    push_u32(&mut command, (words.len() * 4) as u32);
    push_u32(&mut command, 0);
    for word in words {
        push_u32(&mut command, *word);
    }
    command
}

fn copy(destination: u32, destination_x: u32, source: u32, source_x: u32, width: u32) -> Vec<u32> {
    copy_with_box(destination, destination_x, source, source_x, width, 0, 1)
}

fn copy_with_y(
    destination: u32,
    destination_x: u32,
    source: u32,
    source_x: u32,
    y: u32,
) -> Vec<u32> {
    copy_with_box(destination, destination_x, source, source_x, 1, y, 1)
}

fn copy_with_height(
    destination: u32,
    destination_x: u32,
    source: u32,
    source_x: u32,
    height: u32,
) -> Vec<u32> {
    copy_with_box(destination, destination_x, source, source_x, 1, 0, height)
}

fn copy_with_box(
    destination: u32,
    destination_x: u32,
    source: u32,
    source_x: u32,
    width: u32,
    y: u32,
    height: u32,
) -> Vec<u32> {
    vec![
        command_header(17, 0, 13),
        destination,
        0,
        destination_x,
        0,
        0,
        source,
        0,
        source_x,
        y,
        0,
        width,
        height,
        1,
    ]
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
