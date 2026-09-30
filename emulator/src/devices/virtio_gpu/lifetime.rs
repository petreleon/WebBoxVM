//! Allocation identity and retention for accepted asynchronous graphics work.
use super::resource::GpuResource;
use super::VirtioGpu;
use std::collections::HashMap;

mod completion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct ResourceHandle { id: u32, generation: u64 }

#[derive(Clone, Debug, Default)]
pub(super) struct ResourceLifetimes {
    next_generation: u64,
    live: HashMap<u32, ResourceHandle>,
    jobs: HashMap<u32, Vec<ResourceHandle>>,
    pub(super) retired: HashMap<ResourceHandle, GpuResource>,
}

impl VirtioGpu {
    pub(super) fn register_resource_identity(&mut self, id: u32) -> bool {
        let Some(generation) = self.resource_lifetimes.next_generation.checked_add(1) else {
            return false;
        };
        self.resource_lifetimes.next_generation = generation;
        self.resource_lifetimes.live.insert(id, ResourceHandle { id, generation });
        true
    }

    pub(super) fn pin_pending_resources(&mut self, sequence: u32) {
        let Some(effect) = self.pending_3d.last().filter(|job| job.sequence == sequence)
            .and_then(|job| job.effect.as_ref()) else { return; };
        let handles = effect.resource_ids().into_iter()
            .filter_map(|id| self.resource_lifetimes.live.get(&id).copied()).collect();
        self.resource_lifetimes.jobs.insert(sequence, handles);
    }

    pub(super) fn retire_resource(&mut self, id: u32, resource: GpuResource) {
        let handle = self.resource_lifetimes.live.remove(&id).expect("live allocation identity");
        if self.resource_lifetimes.jobs.values().any(|handles| handles.contains(&handle)) {
            // Unref removes guest visibility; accepted jobs retain this exact allocation.
            self.resource_lifetimes.retired.insert(handle, resource);
        } else {
            self.allocated_resource_bytes -= resource.pixels.len();
        }
    }

    pub(super) fn release_pending_resources(&mut self, sequence: u32) {
        let Some(handles) = self.resource_lifetimes.jobs.remove(&sequence) else { return; };
        for handle in handles {
            if !self.resource_lifetimes.jobs.values().any(|handles| handles.contains(&handle)) {
                if let Some(resource) = self.resource_lifetimes.retired.remove(&handle) {
                    self.allocated_resource_bytes -= resource.pixels.len();
                }
            }
        }
    }

    pub(super) fn pending_resource_retired(&self, sequence: u32, id: u32) -> bool {
        self.resource_lifetimes.jobs.get(&sequence).is_some_and(|handles|
            handles.iter().any(|handle| handle.id == id && self.resource_lifetimes.retired.contains_key(handle)))
    }
}
