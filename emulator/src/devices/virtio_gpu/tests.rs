mod backing_many;
mod blob;
mod bounds;
mod commands;
mod context;
mod default_blob;
mod fence;
mod features;
mod host_visible;
mod lifecycle;
mod queue;
mod renderer_blob;
mod three_d;
mod virgl; mod virgl2; mod virgl_singleton_resident;
mod virgl_blend_state;
mod virgl_buffer; mod virgl_buffer_copy;
mod virgl_constant_draw;
mod virgl_copy;
mod virgl_clear_resident;
mod virgl_draw;
mod virgl_draw_capset;
mod virgl_depth_batch;
mod virgl_depth_draw;
mod virgl_depth_textured_draw;
mod virgl_material_batch;
mod virgl_draw_fixture;
mod virgl_dual_texture_draw;
mod virgl_indexed_draw;
mod virgl_queue;
mod virgl_readback;
mod virgl_resident_partial_readback;
mod virgl_rgba_transfer;
mod virgl_sampler_linear;
mod virgl_sampler_repeat;
mod virgl_sampler_state;
mod virgl_solid_batch;
mod virgl_shader_state;
mod virgl_shader_shapes;
mod virgl_split_vertex_draw;
mod virgl_split_texture_draw;
mod virgl_textured_draw;
mod virgl_texture_constant_draw; mod virgl_texture_vertex_uniform_draw;
mod virgl_triangle_primitives;
mod virgl_texture_color_draw;
mod virgl_transfer;
mod virgl_uniform_draw;
mod virgl_vertex_color_draw;
mod virgl_vertex_uniform_draw;
mod virgl_vertex_state;
mod virgl_viewport_state;
use super::protocol::*;
use crate::memory::PhysicalMemory;
use virgl_draw_fixture::word;

impl super::VirtioGpu {
    fn execute_command(&mut self, mem: &mut PhysicalMemory, input: &[u8]) -> Vec<u8> {
        self.execute_queued_command(mem, input).response
    }
}

fn header(command_type: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, command_type);
    push_u32(&mut bytes, super::fence::FLAG_FENCE);
    push_u64(&mut bytes, 0x1122_3344_5566_7788);
    push_u32(&mut bytes, 7);
    push_u32(&mut bytes, 0);
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

pub(super) fn virgl_source_over_state(handle: u32) -> Vec<u32> {
    const SOURCE_OVER: u32 = 1 | (3 << 4) | (19 << 9) | (1 << 17) | (19 << 22) | (15 << 27);
    let mut words = vec![1 | (1 << 8) | (11 << 16), handle, 0, 0, SOURCE_OVER];
    words.extend([0; 7]);
    words.extend([2 | (1 << 8) | (1 << 16), handle]);
    words
}

pub(super) fn virgl_viewport_scissor_state(handle: u32) -> Vec<u32> {
    const RASTERIZER: u32 = (1 << 1) | (1 << 14) | (1 << 29) | (1 << 30);
    [
        vec![
            word(1, 2, 9),
            handle,
            RASTERIZER,
            1.0f32.to_bits(),
            0,
            0,
            1.0f32.to_bits(),
            0,
            0,
            0,
        ],
        vec![word(2, 2, 1), handle],
        vec![
            word(4, 0, 7),
            0,
            256.0f32.to_bits(),
            192.0f32.to_bits(),
            0.5f32.to_bits(),
            512.0f32.to_bits(),
            384.0f32.to_bits(),
            0.5f32.to_bits(),
        ],
        vec![word(15, 0, 3), 0, 448 | (336 << 16), 576 | (432 << 16)],
    ]
    .concat()
}
