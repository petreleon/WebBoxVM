use crate::boot::DEFAULT_BUSYBOX_AARCH64;
use crate::initrd::build_cpio_nodes;

mod archive;
mod modules;

pub(super) struct FastInitrdSpec<'a> {
    pub original: &'a [u8],
    pub kernel_suffix: Option<&'a str>,
    pub root_partition: Option<u32>,
    pub kernel_supported: bool,
    pub root_clean: bool,
}

pub(super) fn build_fast_initrd(spec: FastInitrdSpec<'_>) -> Option<Vec<u8>> {
    if !spec.kernel_supported || !spec.root_clean {
        return None;
    }
    let suffix = spec.kernel_suffix.filter(|suffix| !suffix.is_empty())?;
    let root_partition = spec.root_partition.filter(|partition| *partition > 0)?;
    let module_blobs = modules::extract(spec.original, suffix)?;
    Some(build_cpio_nodes(&archive::nodes(
        DEFAULT_BUSYBOX_AARCH64,
        root_partition,
        module_blobs,
    )))
}

#[cfg(test)]
mod tests;
