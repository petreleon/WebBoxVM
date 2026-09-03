use super::super::protocol::*;
use super::super::resource::*;
use super::super::{SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use super::{append_rect, create_2d, full_scanout, header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

#[test]
fn display_info_is_fixed_and_preserves_request_header() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let response = gpu.execute_command(&mut mem, &header(CMD_GET_DISPLAY_INFO));
    assert_eq!(response.len(), 24 + 16 * 24);
    assert_eq!(response_type(&response), RESP_OK_DISPLAY_INFO);
    assert_eq!(read_u32(&response, 4), Some(1));
    assert_eq!(read_u64(&response, 8), Some(0x1122_3344_5566_7788));
    assert_eq!(read_u32(&response, 16), Some(7));
    assert_eq!(read_u32(&response, 20), Some(9));
    assert_eq!(read_u32(&response, 32), Some(SCANOUT_WIDTH));
    assert_eq!(read_u32(&response, 36), Some(SCANOUT_HEIGHT));
    assert_eq!(read_u32(&response, 40), Some(1));
}

#[test]
fn malformed_and_unknown_commands_return_errors_without_mutation() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_eq!(
        response_type(&gpu.execute_command(&mut mem, &[1, 2, 3])),
        RESP_ERR_UNSPEC
    );
    assert_eq!(
        response_type(&gpu.execute_command(&mut mem, &header(CMD_RESOURCE_CREATE_2D))),
        RESP_ERR_INVALID_PARAMETER
    );
    assert_eq!(
        response_type(&gpu.execute_command(&mut mem, &header(0xffff))),
        RESP_ERR_UNSPEC
    );
    assert!(gpu.resources.is_empty());
}

#[test]
fn command_flow_normalizes_bgrx_and_encodes_damage() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let resource_bytes = (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4) as usize;
    assert_ok(
        &mut gpu,
        &mut mem,
        &create_2d(1, FORMAT_B8G8R8X8_UNORM, SCANOUT_WIDTH, SCANOUT_HEIGHT),
    );
    assert_ok(
        &mut gpu,
        &mut mem,
        &attach(1, RAM_BASE, resource_bytes as u32),
    );
    assert_ok(&mut gpu, &mut mem, &full_scanout(1));

    let pixel_addr = RAM_BASE + ((2 * SCANOUT_WIDTH + 1) * 4) as u64;
    mem.write_bytes(pixel_addr, &[10, 20, 30, 0, 40, 50, 60, 0])
        .unwrap();
    let rect = Rect {
        x: 1,
        y: 2,
        width: 2,
        height: 1,
    };
    let offset = ((2 * SCANOUT_WIDTH + 1) * 4) as u64;
    assert_ok(&mut gpu, &mut mem, &transfer(1, rect, offset));
    assert_ok(&mut gpu, &mut mem, &flush(1, rect));

    let frame = gpu.take_scanout_update();
    assert_eq!(&frame[..4], b"WBGF");
    assert_eq!(read_u32(&frame, 4), Some(1));
    assert_eq!(read_u32(&frame, 8), Some(SCANOUT_WIDTH));
    assert_eq!(read_u32(&frame, 12), Some(SCANOUT_HEIGHT));
    assert_eq!(
        &frame[16..32],
        &[1, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0]
    );
    assert_eq!(&frame[32..], &[10, 20, 30, 255, 40, 50, 60, 255]);
    assert!(gpu.take_scanout_update().is_empty());
}

#[test]
fn flushes_coalesce_to_one_bounding_rectangle() {
    let mut gpu = prepared_gpu();
    let mut mem = PhysicalMemory::new();
    assert_ok(&mut gpu, &mut mem, &flush(1, rect(2, 3, 2, 2)));
    assert_ok(&mut gpu, &mut mem, &flush(1, rect(8, 9, 1, 1)));
    let frame = gpu.take_scanout_update();
    assert_eq!(read_u32(&frame, 16), Some(2));
    assert_eq!(read_u32(&frame, 20), Some(3));
    assert_eq!(read_u32(&frame, 24), Some(7));
    assert_eq!(read_u32(&frame, 28), Some(7));
    assert_eq!(frame.len(), 32 + 7 * 7 * 4);
}

#[test]
fn attach_accepts_one_page_rounded_backing_but_rejects_excess() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_ok(
        &mut gpu,
        &mut mem,
        &create_2d(1, FORMAT_B8G8R8A8_UNORM, 5, 5),
    );
    assert_ok(&mut gpu, &mut mem, &attach(1, RAM_BASE, 4096));

    assert_ok(
        &mut gpu,
        &mut mem,
        &create_2d(2, FORMAT_B8G8R8A8_UNORM, 5, 5),
    );
    assert_eq!(
        response_type(&gpu.execute_command(&mut mem, &attach(2, RAM_BASE, 8192))),
        RESP_ERR_INVALID_PARAMETER
    );
}

fn prepared_gpu() -> VirtioGpu {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_ok(
        &mut gpu,
        &mut mem,
        &create_2d(1, FORMAT_B8G8R8A8_UNORM, SCANOUT_WIDTH, SCANOUT_HEIGHT),
    );
    assert_ok(&mut gpu, &mut mem, &full_scanout(1));
    gpu
}

fn assert_ok(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8]) {
    assert_eq!(
        response_type(&gpu.execute_command(mem, command)),
        RESP_OK_NODATA
    );
}

fn attach(id: u32, addr: u64, len: u32) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut command, id);
    push_u32(&mut command, 1);
    push_u64(&mut command, addr);
    push_u32(&mut command, len);
    push_u32(&mut command, 0);
    command
}

fn transfer(id: u32, rect: Rect, offset: u64) -> Vec<u8> {
    let mut command = header(CMD_TRANSFER_TO_HOST_2D);
    append_rect(&mut command, rect);
    push_u64(&mut command, offset);
    push_u32(&mut command, id);
    push_u32(&mut command, 0);
    command
}

fn flush(id: u32, rect: Rect) -> Vec<u8> {
    let mut command = header(CMD_RESOURCE_FLUSH);
    append_rect(&mut command, rect);
    push_u32(&mut command, id);
    push_u32(&mut command, 0);
    command
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
