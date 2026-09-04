use super::super::VirtioGpu;
use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::resource::FORMAT_B8G8R8A8_UNORM;
use super::super::three_d::CAPSET_ID;
use super::{context_create, create_2d, header, response_type, submit_3d, wbg3_packet};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const USED: u64 = RAM_BASE + 0x1000;
const OLD_RESPONSE: u64 = RAM_BASE + 0x2000;
const NEW_RESPONSE: u64 = RAM_BASE + 0x3000;

#[test]
fn legacy_context_resource_lifecycle_is_noop_but_cannot_submit_wbg3() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let mut legacy = context_create();
    legacy[28..32].copy_from_slice(&0u32.to_le_bytes());
    assert_response(&mut gpu, &mut mem, &legacy, RESP_OK_NODATA);
    assert_response(
        &mut gpu,
        &mut mem,
        &create_2d(1, FORMAT_B8G8R8A8_UNORM, 1, 1),
        RESP_OK_NODATA,
    );
    for command in [CMD_CTX_ATTACH_RESOURCE, CMD_CTX_DETACH_RESOURCE] {
        let mut request = header(command);
        push_u32(&mut request, 1);
        push_u32(&mut request, 0);
        assert_response(&mut gpu, &mut mem, &request, RESP_OK_NODATA);
    }
    assert_response(
        &mut gpu,
        &mut mem,
        &submit_3d(&wbg3_packet(3, 3)),
        RESP_ERR_INVALID_CONTEXT_ID,
    );
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
}

#[test]
fn reused_context_id_has_a_new_completion_timeline() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let old = deferred_submit(&mut gpu, &mut mem);
    attach(&mut gpu, old.sequence, old.header, OLD_RESPONSE, 11);
    assert_response(&mut gpu, &mut mem, &header(CMD_CTX_DESTROY), RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let new = deferred_submit(&mut gpu, &mut mem);
    attach(&mut gpu, new.sequence, new.header, NEW_RESPONSE, 12);
    assert_eq!(read_u32(&gpu.take_3d_update(), 12), Some(old.sequence));
    assert_eq!(read_u32(&gpu.take_3d_update(), 12), Some(new.sequence));

    assert!(gpu.complete_3d(&mut mem, new.sequence, true));
    assert_eq!(gpu.contexts.get(&7), Some(&CAPSET_ID));
    assert_eq!(gpu.pending_3d.len(), 1);
    assert!(gpu.complete_3d(&mut mem, old.sequence, true));
    assert_eq!(mem.read(USED + 2, 2), Some(2));
    assert_eq!(mem.read(OLD_RESPONSE, 4), Some(RESP_OK_NODATA as u64));
    assert_eq!(mem.read(NEW_RESPONSE, 4), Some(RESP_OK_NODATA as u64));
}

#[test]
fn device_reset_does_not_reuse_an_inflight_browser_sequence() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    assert_eq!(gpu.reset_generation(), 0);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let old = deferred_submit(&mut gpu, &mut mem);
    assert!(!gpu.write(&mut mem, 0x070, 0, 4));
    assert_eq!(gpu.reset_generation(), 1);
    assert_response(&mut gpu, &mut mem, &context_create(), RESP_OK_NODATA);
    let new = deferred_submit(&mut gpu, &mut mem);
    assert_ne!(old.sequence, new.sequence);
    attach(&mut gpu, new.sequence, new.header, NEW_RESPONSE, 12);
    assert_eq!(read_u32(&gpu.take_3d_update(), 12), Some(new.sequence));
    assert!(!gpu.complete_3d(&mut mem, old.sequence, true));
    assert_eq!(gpu.pending_3d.len(), 1);
    assert_eq!(gpu.pending_3d[0].sequence, new.sequence);
}

fn deferred_submit(
    gpu: &mut VirtioGpu,
    mem: &mut PhysicalMemory,
) -> super::super::three_d::DeferredSubmit {
    gpu.execute_queued_command(mem, &submit_3d(&wbg3_packet(3, 3)))
        .deferred
        .expect("valid WBG3 submission must defer")
}

fn attach(gpu: &mut VirtioGpu, sequence: u32, header: CtrlHeader, output: u64, head: u16) {
    assert!(gpu.attach_3d_completion(
        sequence,
        PendingCompletion {
            header,
            output: vec![WritableRegion {
                addr: output,
                len: 24
            }],
            used: USED,
            queue_size: 8,
            head,
        }
    ));
}

fn assert_response(gpu: &mut VirtioGpu, mem: &mut PhysicalMemory, command: &[u8], expected: u32) {
    assert_eq!(response_type(&gpu.execute_command(mem, command)), expected);
}
