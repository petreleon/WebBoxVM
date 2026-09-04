use super::clear::Clear;
use super::super::DrawWork;
use super::super::draw::MAX_VIRGL_BATCH_DRAWS;
use super::super::super::DeferredSubmit;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER};

#[derive(Default)]
pub(super) struct Batch {
    works: Vec<DrawWork>,
}

impl Batch {
    pub(super) fn is_empty(&self) -> bool {
        self.works.is_empty()
    }

    pub(super) fn push(&mut self, work: DrawWork) -> Result<(), u32> {
        if self.works.len() == MAX_VIRGL_BATCH_DRAWS {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        self.works.push(work);
        Ok(())
    }
}

pub(super) fn deferred(
    gpu: &mut VirtioGpu,
    header: CtrlHeader,
    generation: u32,
    clear: Option<Clear>,
    batch: Batch,
) -> Result<Option<DeferredSubmit>, u32> {
    let works = batch.works;
    match clear {
        Some(Clear { resource, color, rect, .. }) if works.len() == 1 => Ok(Some(
            gpu.queue_virgl_draw(header, generation, resource, rect, color, works.into_iter().next().unwrap())?,
        )),
        Some(Clear { resource, color, rect, .. }) if works.len() > 1 => Ok(Some(
            gpu.queue_virgl_batch(header, generation, resource, rect, color, works)?,
        )),
        Some(Clear { resource, depth_resource: None, color, rect }) if works.is_empty() => Ok(Some(
            gpu.queue_virgl_clear(header, generation, resource, rect, color)?,
        )),
        None if works.is_empty() => Ok(None),
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}
