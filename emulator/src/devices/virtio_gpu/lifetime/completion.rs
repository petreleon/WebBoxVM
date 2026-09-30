use super::super::VirtioGpu;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn with_pending_resources(
        &mut self, sequence: u32, apply: impl FnOnce(&mut Self) -> bool,
    ) -> bool {
        let handles = self.resource_lifetimes.jobs.get(&sequence).cloned().unwrap_or_default();
        let mut restored = Vec::new();
        let mut owners = Vec::new();
        for handle in handles {
            let retained_owner = self.resource_lifetimes.retained_residents.remove(&handle);
            if retained_owner.is_some() || self.resource_lifetimes.retired.contains_key(&handle) {
                let live_owner = self.resident_resources.remove(&handle.id);
                if let Some(owner) = retained_owner { self.resident_resources.insert(handle.id, owner); }
                owners.push((handle, live_owner));
            }
            if let Some(retired) = self.resource_lifetimes.retired.remove(&handle) {
                // This synchronous mutable borrow cannot admit new guest work. Temporarily
                // resolve raw protocol IDs to the pinned allocation, then restore live IDs.
                let live = self.resources.insert(handle.id, retired);
                restored.push((handle, live));
            }
        }
        let hide_scanout = self.scanout.is_some_and(|scanout|
            restored.iter().any(|(handle, _)| handle.id == scanout.resource_id));
        let scanout = if hide_scanout { self.scanout.take() } else { None };
        let result = apply(self);
        if hide_scanout { self.scanout = scanout; }
        for (handle, live_owner) in owners {
            if let Some(owner) = self.resident_resources.remove(&handle.id) {
                self.resource_lifetimes.retained_residents.insert(handle, owner);
            }
            if let Some(owner) = live_owner { self.resident_resources.insert(handle.id, owner); }
        }
        for (handle, live) in restored {
            let retired = self.resources.remove(&handle.id).expect("pinned allocation retained");
            self.resource_lifetimes.retired.insert(handle, retired);
            if let Some(resource) = live { self.resources.insert(handle.id, resource); }
        }
        result
    }
}
