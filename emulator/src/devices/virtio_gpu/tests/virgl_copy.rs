use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::memory::PhysicalMemory;
const WIDTH: u32 = 4;
const HEIGHT: u32 = 3;

#[test]
fn resource_copy_region_copies_an_attached_offscreen_rectangle() {
    let (mut gpu, mut mem) = prepared_gpu();
    for (index, pixel) in gpu
        .resources
        .get_mut(&1)
        .unwrap()
        .pixels
        .chunks_exact_mut(4)
        .enumerate()
    {
        pixel.copy_from_slice(&[index as u8, index as u8 + 20, index as u8 + 40, 255]);
    }

    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&copy(2, 2, 1, 1, 1, 0, 2, 2)),
        RESP_OK_NODATA,
    );

    let target = &gpu.resources[&2].pixels;
    assert_eq!(&target[24..32], &[1, 21, 41, 255, 2, 22, 42, 255]);
    assert_eq!(&target[40..48], &[5, 25, 45, 255, 6, 26, 46, 255]);
    assert!(
        target[..24]
            .iter()
            .chain(target[32..40].iter())
            .all(|byte| *byte == 0)
    );
}

#[test]
fn resource_copy_region_uses_a_source_snapshot_for_self_overlap() {
    let (mut gpu, mut mem) = prepared_gpu();
    gpu.resources.get_mut(&1).unwrap().pixels[..16]
        .copy_from_slice(&[1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255]);

    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&copy(1, 1, 0, 1, 0, 0, 3, 1)),
        RESP_OK_NODATA,
    );
    assert_eq!(
        &gpu.resources[&1].pixels[..16],
        &[1, 0, 0, 255, 1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255]
    );
}

#[test]
fn resource_copy_region_rejects_invalid_streams_without_mutating_resources() {
    let (mut gpu, mut mem) = prepared_gpu();
    gpu.resources.get_mut(&1).unwrap().pixels[..4].copy_from_slice(&[9; 4]);
    let before = gpu.resources[&2].pixels.clone();
    let mut malformed = copy(2, 0, 0, 1, 0, 0, 1, 1);
    malformed.push(command_header(255, 0, 0));

    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&malformed),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(gpu.resources[&2].pixels, before);
    assert_response(
        &mut gpu,
        &mut mem,
        &context_resource(CMD_CTX_DETACH_RESOURCE, 2),
        RESP_OK_NODATA,
    );
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&copy(2, 0, 0, 1, 0, 0, 1, 1)),
        RESP_ERR_INVALID_PARAMETER,
    );
    assert_eq!(gpu.resources[&2].pixels, before);
}

fn prepared_gpu() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    for resource in [1, 2] {
        assert_response(
            &mut gpu,
            &mut mem,
            &resource_create(resource),
            RESP_OK_NODATA,
        );
    }
    assert_response(&mut gpu, &mut mem, &virgl_context(), RESP_OK_NODATA);
    for resource in [1, 2] {
        assert_response(
            &mut gpu,
            &mut mem,
            &context_resource(CMD_CTX_ATTACH_RESOURCE, resource),
            RESP_OK_NODATA,
        );
    }
    (gpu, mem)
}

fn resource_create(resource: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_CREATE_3D);
    for value in [resource, 2, 1, 2, WIDTH, HEIGHT, 1, 1, 0, 1, 0, 0] {
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

fn copy(
    destination: u32,
    destination_x: u32,
    destination_y: u32,
    source: u32,
    source_x: u32,
    source_y: u32,
    width: u32,
    height: u32,
) -> Vec<u32> {
    vec![
        command_header(17, 0, 13),
        destination,
        0,
        destination_x,
        destination_y,
        0,
        source,
        0,
        source_x,
        source_y,
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
