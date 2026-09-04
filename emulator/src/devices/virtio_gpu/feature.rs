use super::{
    VIRTIO_F_VERSION_1, VIRTIO_GPU_F_CONTEXT_INIT, VIRTIO_GPU_F_RESOURCE_BLOB,
    VIRTIO_GPU_F_VIRGL, VirtioGpu,
};

pub(super) const STATUS_FEATURES_OK: u32 = 1 << 3;

impl VirtioGpu {
    pub(super) fn selected_device_features(&self) -> u64 {
        let features = self.offered_features();
        match self.device_features_sel {
            0 => features & u64::from(u32::MAX),
            1 => features >> 32,
            _ => 0,
        }
    }

    pub(super) fn write_driver_features(&mut self, value: u32) {
        if let Some(slot) = self
            .driver_features
            .get_mut(self.driver_features_sel as usize)
        {
            *slot = value;
        }
        self.revalidate_features();
    }

    pub(super) fn write_status(&mut self, value: u32) {
        if value == 0 {
            self.cold_reset();
            return;
        }
        self.status = value;
        self.revalidate_features();
    }

    pub(super) fn feature_enabled(&self, feature: u64) -> bool {
        self.status & STATUS_FEATURES_OK != 0 && self.driver_features() & feature == feature
    }

    fn revalidate_features(&mut self) {
        if self.status & STATUS_FEATURES_OK != 0 && !self.features_are_valid() {
            self.status &= !STATUS_FEATURES_OK;
        }
    }

    fn offered_features(&self) -> u64 {
        VIRTIO_F_VERSION_1
            | VIRTIO_GPU_F_VIRGL
            | VIRTIO_GPU_F_RESOURCE_BLOB
            | VIRTIO_GPU_F_CONTEXT_INIT
    }

    fn driver_features(&self) -> u64 {
        u64::from(self.driver_features[0]) | (u64::from(self.driver_features[1]) << 32)
    }

    fn features_are_valid(&self) -> bool {
        let requested = self.driver_features();
        requested & !self.offered_features() == 0 && requested & VIRTIO_F_VERSION_1 != 0
    }
}
