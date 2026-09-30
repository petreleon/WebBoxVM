use super::super::super::completion::{PendingCompletion, WritableRegion};
use super::super::super::protocol::*;
use super::super::virgl_draw_fixture::*;
use super::super::virgl_solid_batch::{configure, constants};
use super::super::header;
use crate::constants::RAM_BASE;

#[test]
fn late_gpu_readback_uses_retired_dimensions_and_preserves_new_resource() {
    let (mut gpu, mut mem) = prepared();
    configure(&mut gpu, &mut mem);
    let mut words = clear([0.0, 0.0, 0.0, 1.0]);
    words.extend(constants([1.0, 0.0, 0.0, 0.5])); words.extend(draw());
    words.extend(constants([0.0, 1.0, 0.0, 0.5])); words.extend(draw());
    let pending = gpu.execute_queued_command(&mut mem, &submit(&words)).deferred.unwrap();
    assert!(gpu.attach_3d_completion(pending.sequence, PendingCompletion {
        header: pending.header,
        output: vec![WritableRegion { addr: RAM_BASE + 0x2000, len: 24 }],
        used: RAM_BASE + 0x1000, queue_size: 8, head: 0,
    }));
    assert_eq!(gpu.take_3d_update().get(..4), Some(&b"VGB1"[..]));
    let mut unref = header(CMD_RESOURCE_UNREF);
    push_u32(&mut unref, TARGET); push_u32(&mut unref, 0);
    assert_response(&mut gpu, &mut mem, &unref, RESP_OK_NODATA);
    assert_response(&mut gpu, &mut mem, &create(TARGET, 2, 1, 2, 8, 8), RESP_OK_NODATA);
    gpu.resources.get_mut(&TARGET).unwrap().pixels.fill(29);

    let pixels = [1, 2, 3, 255].repeat(1024 * 768);
    assert!(gpu.complete_3d_readback(&mut mem, pending.sequence, 2, &pixels));
    assert_eq!(mem.read(RAM_BASE + 0x2000, 4), Some(RESP_OK_NODATA as u64));
    assert!(gpu.resources[&TARGET].pixels.iter().all(|byte| *byte == 29));
    assert!(!gpu.resident_resources.contains_key(&TARGET));
    assert!(gpu.resource_lifetimes.retired.is_empty());
    assert!(gpu.take_scanout_update().is_empty());
}
