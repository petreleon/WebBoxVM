use super::super::VirtioGpu;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn with_pending_resources(
        &mut self, sequence: u32, apply: impl FnOnce(&mut Self) -> bool,
    ) -> bool {
        let handles = self.resource_lifetimes.jobs.get(&sequence).cloned().unwrap_or_default();
        let mut restored = Vec::new();
        for handle in handles {
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
        for (handle, live) in restored {
            let retired = self.resources.remove(&handle.id).expect("pinned allocation retained");
            self.resource_lifetimes.retired.insert(handle, retired);
            if let Some(resource) = live { self.resources.insert(handle.id, resource); }
        }
        result
    }
}
