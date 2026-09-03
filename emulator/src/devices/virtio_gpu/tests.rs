mod backing_many;
mod bounds;
mod commands;
mod context;
mod lifecycle;
mod queue;
mod three_d;
mod virgl;
mod virgl_queue;
mod virgl_transfer;

use super::protocol::*;
use crate::memory::PhysicalMemory;

impl super::VirtioGpu {
    fn execute_command(&mut self, mem: &PhysicalMemory, input: &[u8]) -> Vec<u8> {
        self.execute_queued_command(mem, input).response
    }
}

fn header(command_type: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, command_type);
    push_u32(&mut bytes, 1);
    push_u64(&mut bytes, 0x1122_3344_5566_7788);
    push_u32(&mut bytes, 7);
    push_u32(&mut bytes, 9);
    bytes
}

fn response_type(bytes: &[u8]) -> u32 {
    read_u32(bytes, 0).expect("response type")
}

fn append_rect(bytes: &mut Vec<u8>, rect: Rect) {
    for value in [rect.x, rect.y, rect.width, rect.height] {
        push_u32(bytes, value);
    }
}

fn create_2d(resource_id: u32, format: u32, width: u32, height: u32) -> Vec<u8> {
    let mut bytes = header(CMD_RESOURCE_CREATE_2D);
    for value in [resource_id, format, width, height] {
        push_u32(&mut bytes, value);
    }
    bytes
}

fn full_scanout(resource_id: u32) -> Vec<u8> {
    let mut bytes = header(CMD_SET_SCANOUT);
    append_rect(
        &mut bytes,
        Rect {
            x: 0,
            y: 0,
            width: super::SCANOUT_WIDTH,
            height: super::SCANOUT_HEIGHT,
        },
    );
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, resource_id);
    bytes
}

fn context_create() -> Vec<u8> {
    let mut bytes = header(CMD_CTX_CREATE);
    push_u32(&mut bytes, 4);
    push_u32(&mut bytes, super::three_d::CAPSET_ID);
    bytes.extend_from_slice(b"test");
    bytes.resize(96, 0);
    bytes
}

fn submit_3d(packet: &[u8]) -> Vec<u8> {
    let mut bytes = header(CMD_SUBMIT_3D);
    push_u32(&mut bytes, packet.len() as u32);
    push_u32(&mut bytes, 0);
    bytes.extend_from_slice(packet);
    bytes
}

fn wbg3_packet(vertices: u32, indices: u32) -> Vec<u8> {
    let mut packet = b"WBG3".to_vec();
    for value in [1, 1, 99, 640, 480, vertices, indices] {
        push_u32(&mut packet, value);
    }
    for value in [0.1f32, 0.2, 0.3, 1.0] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for index in 0..16 {
        let value = if index % 5 == 0 { 1.0f32 } else { 0.0 };
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for _ in 0..vertices * 7 {
        packet.extend_from_slice(&0.25f32.to_le_bytes());
    }
    for index in 0..indices {
        packet.extend_from_slice(&((index % vertices.max(1)) as u16).to_le_bytes());
    }
    packet
}
