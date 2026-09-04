use super::VirglContext;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn set_fragment_constants(
        &mut self,
        values: Option<[u32; 4]>,
    ) {
        self.fragment_constants = values;
    }
}
