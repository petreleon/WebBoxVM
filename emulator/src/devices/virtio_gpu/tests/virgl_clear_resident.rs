use super::super::completion::{PendingCompletion, WritableRegion};
use super::super::protocol::*;
use super::super::three_d::BrowserCompletion;
use super::virgl_draw_fixture::*;
use crate::constants::RAM_BASE;

#[test]
fn full_clear_promotes_and_rekeys_one_bounded_resident_target() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let first = gpu.execute_queued_command(&mut mem, &submit(&clear([0.25, 0.5, 0.75, 1.0])))
        .deferred.expect("first clear");
    attach(&mut gpu, first.sequence, first.header, RAM_BASE + 0x7000, 1);
    assert_eq!(gpu.pending_3d[0].browser_completion, BrowserCompletion::Resident);
    let packet = gpu.take_3d_update();
    assert_eq!(&packet[..4], b"VGC1");
    assert_eq!([4, 36].map(|offset| read_u32(&packet, offset)), [Some(2), Some(0)]);
    assert_eq!(packet.len(), 40);
    assert!(gpu.complete_3d_resident(&mut mem, first.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, first.sequence);
    assert_eq!(&gpu.resources[&TARGET].pixels[..4], &[0, 0, 0, 0]);

    let second = gpu.execute_queued_command(&mut mem, &submit(&clear([0.5, 0.25, 0.0, 1.0])))
        .deferred.expect("replacement clear");
    attach(&mut gpu, second.sequence, second.header, RAM_BASE + 0x7100, 2);
    let packet = gpu.take_3d_update();
    assert_eq!([4, 36].map(|offset| read_u32(&packet, offset)), [Some(2), Some(first.sequence)]);
    assert!(gpu.complete_3d_resident(&mut mem, second.sequence));
    assert_eq!(gpu.resident_resources[&TARGET].producer_sequence, second.sequence);
    assert!(gpu.take_3d_update().is_empty());
}

#[test]
fn cpu_authority_epoch_rejects_an_unresolved_resident_clear() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let deferred = gpu.execute_queued_command(&mut mem, &submit(&clear([0.0, 0.0, 0.0, 1.0])))
        .deferred.expect("resident clear");
    attach(&mut gpu, deferred.sequence, deferred.header, RAM_BASE + 0x7000, 1);
    let _ = gpu.take_3d_update();
    gpu.forget_resident(TARGET);
    assert!(gpu.complete_3d_resident(&mut mem, deferred.sequence));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    assert_eq!(read_u32(&gpu.take_3d_update(), 8), Some(deferred.sequence));
    assert_eq!(mem.read(RAM_BASE + 0x7000, 4), Some(RESP_ERR_UNSPEC as u64));
}

fn configure(gpu: &mut super::super::VirtioGpu, mem: &mut crate::memory::PhysicalMemory) {
    let mut state = surface_create(9, TARGET);
    state.extend(framebuffer(9));
    assert_response(gpu, mem, &submit(&state), RESP_OK_NODATA);
}

fn attach(gpu: &mut super::super::VirtioGpu, sequence: u32, header: CtrlHeader, address: u64, head: u16) {
    assert!(gpu.attach_3d_completion(sequence, PendingCompletion {
        header, output: vec![WritableRegion { addr: address, len: 24 }], used: RAM_BASE + 0x7200,
        queue_size: 8, head,
    }));
}
