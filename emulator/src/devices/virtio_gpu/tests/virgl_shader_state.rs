use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::{ShaderKind, ShaderProgram, VIRGL_CAPSET_ID};
use super::{header, response_type};
use crate::memory::PhysicalMemory;

const CONTEXT_ID: u32 = 7;
const VERT: &str = "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
const FRAG: &str =
    "FRAG\nDCL OUT[0], COLOR\nIMM[0] FLT32 {0x3f800000, 0, 0, 1}\n0: MOV OUT[0], IMM[0]\n1: END\n";

#[test]
fn standard_shader_objects_bind_unbind_and_release() {
    let (mut gpu, mut mem) = prepared();
    let mut words = shader_create(11, 0, VERT);
    words.extend(shader_create(12, 4, FRAG));
    words.extend(bind(11, 0));
    words.extend(bind(12, 4));
    assert_response(&mut gpu, &mut mem, &submit(&words), RESP_OK_NODATA);
    assert_eq!(
        bound(&gpu, ShaderKind::Vertex),
        Some(ShaderProgram::VertexPassthrough)
    );
    assert_eq!(
        bound(&gpu, ShaderKind::Fragment),
        Some(ShaderProgram::FragmentSolid([
            0x3f80_0000,
            0,
            0,
            0x3f80_0000,
        ]))
    );
    let mut padded = shader_create(13, 0, VERT);
    let padding_shift = (padded[3] as usize % 4) * 8;
    assert_ne!(padding_shift, 0);
    *padded.last_mut().expect("shader text word") |= 0xa5u32 << padding_shift;
    padded.extend(bind(13, 0));
    assert_response(&mut gpu, &mut mem, &submit(&padded), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &submit(&bind(0, 0)), RESP_OK_NODATA);
    assert_eq!(bound(&gpu, ShaderKind::Vertex), None);
    assert_response(&mut gpu, &mut mem, &submit(&destroy(12)), RESP_OK_NODATA);
    assert_eq!(bound(&gpu, ShaderKind::Fragment), None);
}

#[test]
fn shader_stream_rejection_is_transactional_and_type_safe() {
    let (mut gpu, mut mem) = prepared();
    let mut initial = shader_create(11, 0, VERT);
    initial.extend(bind(11, 0));
    assert_response(&mut gpu, &mut mem, &submit(&initial), RESP_OK_NODATA);
    let mut continuation = shader_create(12, 0, VERT);
    continuation[3] |= 1 << 31;
    let mut partial = shader_create(12, 0, VERT);
    partial.extend(shader_create(13, 4, "FRAG\nDCL OUT[0], COLOR\nEND\n"));
    for words in [
        shader_create(12, 1, FRAG),
        shader_create(12, 0, "VERT\nDCL IN[0]\nEND\n"),
        continuation,
        bind(11, 4),
        bind(99, 0),
        partial,
    ] {
        assert_response(
            &mut gpu,
            &mut mem,
            &submit(&words),
            RESP_ERR_INVALID_PARAMETER,
        );
    }
    assert_eq!(
        bound(&gpu, ShaderKind::Vertex),
        Some(ShaderProgram::VertexPassthrough)
    );
    assert_eq!(bound(&gpu, ShaderKind::Fragment), None);
    assert_response(
        &mut gpu,
        &mut mem,
        &submit(&bind(12, 0)),
        RESP_ERR_INVALID_PARAMETER,
    );
}

fn prepared() -> (VirtioGpu, PhysicalMemory) {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let mut command = header(CMD_CTX_CREATE);
    push_u32(&mut command, 5);
    push_u32(&mut command, VIRGL_CAPSET_ID);
    command.extend_from_slice(b"virgl");
    command.resize(96, 0);
    assert_response(&mut gpu, &mut mem, &command, RESP_OK_NODATA);
    (gpu, mem)
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

fn shader_create(handle: u32, kind: u32, source: &str) -> Vec<u32> {
    let mut bytes = source.as_bytes().to_vec();
    bytes.push(0);
    let mut words = vec![
        command_header(1, 4, (5 + bytes.len().div_ceil(4)) as u16),
        handle,
        kind,
        bytes.len() as u32,
        8,
        0,
    ];
    for bytes in bytes.chunks(4) {
        let mut word = [0; 4];
        word[..bytes.len()].copy_from_slice(bytes);
        words.push(u32::from_le_bytes(word));
    }
    words
}

fn bind(handle: u32, kind: u32) -> Vec<u32> {
    vec![command_header(29, 0, 2), handle, kind]
}

fn destroy(handle: u32) -> Vec<u32> {
    vec![command_header(3, 4, 1), handle]
}

fn bound(gpu: &VirtioGpu, kind: ShaderKind) -> Option<ShaderProgram> {
    gpu.virgl_contexts[&CONTEXT_ID].bound_shader(kind)
}

fn command_header(command: u8, object: u8, length: u16) -> u32 {
    u32::from(command) | (u32::from(object) << 8) | (u32::from(length) << 16)
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
