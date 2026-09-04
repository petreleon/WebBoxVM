use super::{VirtioGpu, assert_response, create, header, prepared};
use super::super::super::protocol::{CMD_CTX_ATTACH_RESOURCE, RESP_OK_NODATA, push_u32};
use super::super::super::three_d::ResidentResource;
use crate::memory::PhysicalMemory;

pub(crate) fn prepared_nonresident() -> (VirtioGpu, PhysicalMemory) {
    let (mut gpu, mut mem) = prepared();
    let generation = gpu.virgl_contexts[&7].generation;
    for id in 100..116 {
        assert_response(&mut gpu, &mut mem, &create(id, 2, 1, 2, 1, 1), RESP_OK_NODATA);
        let mut attach = header(CMD_CTX_ATTACH_RESOURCE);
        for value in [id, 0] { push_u32(&mut attach, value); }
        assert_response(&mut gpu, &mut mem, &attach, RESP_OK_NODATA);
        gpu.resident_resources.insert(id, ResidentResource { context_id: 7, generation, producer_sequence: id });
    }
    (gpu, mem)
}
