use super::Pending3dEffect;

impl Pending3dEffect {
    pub(in crate::devices::virtio_gpu) fn resource_ids(&self) -> Vec<u32> {
        let mut ids = match self {
            Self::VirglClear { resource_id, .. } | Self::VirglBatch { resource_id, .. }
            | Self::VirglResidentReadback { resource_id, .. } => vec![*resource_id],
            Self::VirglDraw { resource_id, depth_resource, .. } => {
                let mut ids = vec![*resource_id];
                ids.extend(*depth_resource);
                ids
            }
            Self::VirglDepthBatch { resource_id, depth_resource, .. } => vec![*resource_id, *depth_resource],
            Self::VirglResidentCopy { resource_id, source_resource_id, .. } => vec![*resource_id, *source_resource_id],
        };
        if let Self::VirglBatch { works, .. } | Self::VirglDepthBatch { works, .. } = self {
            ids.extend(works.iter().filter_map(|work| work.resident_texture().map(|texture| texture.resource_id)));
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}
