use super::super::protocol::*;
use super::super::three_d::packet::MAX_WBG3_PACKET_BYTES;
use super::super::three_d::{MAX_3D_INDICES, MAX_3D_VERTICES};
use super::super::{QUEUE_NUM_MAX, VirtioGpu};
use super::{context_create, header, response_type, submit_3d, wbg3_packet};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const DESC: u64 = RAM_BASE + 0x1000;
const AVAIL: u64 = RAM_BASE + 0x3000;
const USED: u64 = RAM_BASE + 0x4000;
const REQUEST: u64 = RAM_BASE + 0x5000;
const RESPONSE: u64 = RAM_BASE + 0x30000;

#[test]
fn split_control_queue_completes_response_and_interrupt() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    configure_queue(&mut gpu, &mut mem, 0, &header(CMD_GET_DISPLAY_INFO), 512);
    assert!(gpu.write(&mut mem, 0x050, 0, 4));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_OK_DISPLAY_INFO as u64));
    assert_eq!(mem.read(USED + 2, 2), Some(1));
    assert_eq!(mem.read(USED + 4, 4), Some(0));
    assert_eq!(mem.read(USED + 8, 4), Some((24 + 16 * 24) as u64));
    assert_eq!(gpu.read(0x060, 4), Some(1));
    gpu.write(&mut mem, 0x060, 0, 4);
    assert_eq!(gpu.read(0x060, 4), Some(1));
    gpu.write(&mut mem, 0x064, 1, 4);
    assert_eq!(gpu.read(0x060, 4), Some(0));
}

#[test]
fn short_response_buffer_is_consumed_without_out_of_bounds_write() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    configure_queue(&mut gpu, &mut mem, 0, &header(CMD_GET_DISPLAY_INFO), 8);
    assert!(gpu.write(&mut mem, 0x050, 0, 4));
    assert_eq!(mem.read(USED + 8, 4), Some(0));
    assert_eq!(mem.read(RESPONSE, 8), Some(0));
}

#[test]
fn both_queues_are_exposed_and_cursor_queue_rejects_unknown_commands() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    gpu.write(&mut mem, 0x030, 1, 4);
    assert_eq!(gpu.read(0x034, 4), Some(QUEUE_NUM_MAX as u64));
    configure_queue(&mut gpu, &mut mem, 1, &header(0x0300), 24);
    assert!(gpu.write(&mut mem, 0x050, 1, 4));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_ERR_UNSPEC as u64));
}

#[test]
fn maximum_wbg3_submit_defers_used_ring_until_success_ack() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_eq!(
        response_type(&gpu.execute_command(&mem, &context_create())),
        RESP_OK_NODATA
    );
    let packet = wbg3_packet(MAX_3D_VERTICES, MAX_3D_INDICES);
    assert_eq!(packet.len(), MAX_WBG3_PACKET_BYTES);
    configure_queue(&mut gpu, &mut mem, 0, &submit_3d(&packet), 24);
    assert!(!gpu.write(&mut mem, 0x050, 0, 4));
    assert_eq!(mem.read(USED + 2, 2), Some(0));
    assert_eq!(gpu.read(0x060, 4), Some(0));
    assert!(!gpu.complete_3d(&mut mem, 1, true));
    let exported = gpu.take_3d_update();
    let sequence = read_u32(&exported, 12).unwrap();
    assert_eq!(exported.len(), MAX_WBG3_PACKET_BYTES);
    assert!(gpu.complete_3d(&mut mem, sequence, true));
    assert_eq!(mem.read(USED + 2, 2), Some(1));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_OK_NODATA as u64));
    assert_eq!(gpu.read(0x060, 4), Some(1));
    assert!(!gpu.complete_3d(&mut mem, sequence, true));
}

#[test]
fn failed_wbg3_ack_completes_with_error() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    gpu.execute_command(&mem, &context_create());
    configure_queue(&mut gpu, &mut mem, 0, &submit_3d(&wbg3_packet(3, 3)), 24);
    assert!(!gpu.write(&mut mem, 0x050, 0, 4));
    let packet = gpu.take_3d_update();
    assert!(gpu.complete_3d(&mut mem, read_u32(&packet, 12).unwrap(), false));
    assert_eq!(mem.read(RESPONSE, 4), Some(RESP_ERR_UNSPEC as u64));
}

fn configure_queue(
    gpu: &mut VirtioGpu,
    mem: &mut PhysicalMemory,
    queue: u32,
    request: &[u8],
    response_len: u32,
) {
    mem.write_bytes(REQUEST, request).unwrap();
    write_desc(mem, DESC, REQUEST, request.len() as u32, 1, 1);
    write_desc(mem, DESC + 16, RESPONSE, response_len, 2, 0);
    mem.write(AVAIL + 2, 2, 1).unwrap();
    mem.write(AVAIL + 4, 2, 0).unwrap();
    gpu.write(mem, 0x030, queue as u64, 4);
    gpu.write(mem, 0x038, 8, 4);
    set_addr(gpu, mem, 0x080, DESC);
    set_addr(gpu, mem, 0x090, AVAIL);
    set_addr(gpu, mem, 0x0a0, USED);
    gpu.write(mem, 0x044, 1, 4);
}

fn set_addr(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, offset: u64, addr: u64) {
    gpu.write(mem, offset, addr as u32 as u64, 4);
    gpu.write(mem, offset + 4, addr >> 32, 4);
}

fn write_desc(mem: &mut PhysicalMemory, base: u64, addr: u64, len: u32, flags: u16, next: u16) {
    mem.write(base, 8, addr).unwrap();
    mem.write(base + 8, 4, len as u64).unwrap();
    mem.write(base + 12, 2, flags as u64).unwrap();
    mem.write(base + 14, 2, next as u64).unwrap();
}
