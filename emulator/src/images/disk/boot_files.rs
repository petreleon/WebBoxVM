use super::ext4_partition::PartitionReader;
use super::partitions::Partition;
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;
use ext4_view::Ext4;

#[derive(Clone, Debug)]
pub struct InstalledDiskBoot {
    pub disk: SparseDiskSnapshot,
    pub kernel: Vec<u8>,
    pub initrd: Vec<u8>,
    pub bootargs: String,
    pub boot_partition: u32,
    pub root_partition: Option<u32>,
}

pub fn extract_installed_boot(
    disk: SparseDiskSnapshot,
    partitions: &[Partition],
) -> Result<InstalledDiskBoot, String> {
    let mut root = None;
    let mut boot = None;
    for partition in partitions {
        let Ok(fs) = load_ext4(disk.clone(), *partition) else {
            continue;
        };
        if root.is_none() && fs.exists("/etc/fstab").unwrap_or(false) {
            root = Some((partition.number, fs.uuid().to_string()));
        }
        if boot.is_none() {
            if let Some((kernel, initrd)) = read_boot_pair(&fs) {
                boot = Some((*partition, kernel, initrd));
            }
        }
    }

    let (partition, kernel, initrd) =
        boot.ok_or_else(|| "installed disk has no readable kernel/initrd pair".to_string())?;
    let bootargs = bootargs(root.as_ref(), partition.number);
    Ok(InstalledDiskBoot {
        disk,
        kernel,
        initrd,
        bootargs,
        boot_partition: partition.number,
        root_partition: root.map(|(number, _)| number),
    })
}

fn load_ext4(disk: SparseDiskSnapshot, partition: Partition) -> Result<Ext4, String> {
    let reader = PartitionReader::new(disk, partition)?;
    Ext4::load(Box::new(reader)).map_err(|err| err.to_string())
}

fn read_boot_pair(fs: &Ext4) -> Option<(Vec<u8>, Vec<u8>)> {
    for (kernel, initrd) in [
        ("/vmlinuz", "/initrd.img"),
        ("/boot/vmlinuz", "/boot/initrd.img"),
    ] {
        if let Ok(pair) = read_pair(fs, kernel, initrd) {
            return Some(pair);
        }
    }
    for dir in ["/", "/boot"] {
        if let Some((kernel, initrd)) = select_versioned_pair(&list_dir(fs, dir), dir) {
            if let Ok(pair) = read_pair(fs, &kernel, &initrd) {
                return Some(pair);
            }
        }
    }
    None
}

fn read_pair(fs: &Ext4, kernel: &str, initrd: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let kernel = fs.read(kernel).map_err(|err| err.to_string())?;
    let initrd = fs.read(initrd).map_err(|err| err.to_string())?;
    Ok((kernel, initrd))
}

fn list_dir(fs: &Ext4, dir: &str) -> Vec<String> {
    let Ok(entries) = fs.read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().as_str().ok().map(ToString::to_string))
        .collect()
}

fn select_versioned_pair(names: &[String], dir: &str) -> Option<(String, String)> {
    let mut kernels: Vec<_> = names
        .iter()
        .filter_map(|name| name.strip_prefix("vmlinuz-"))
        .collect();
    kernels.sort_unstable();
    kernels.reverse();
    for suffix in kernels {
        let initrd = format!("initrd.img-{suffix}");
        if names.iter().any(|name| name == &initrd) {
            return Some((
                join_path(dir, &format!("vmlinuz-{suffix}")),
                join_path(dir, &initrd),
            ));
        }
    }
    None
}

fn join_path(dir: &str, name: &str) -> String {
    if dir == "/" {
        format!("/{name}")
    } else {
        format!("{dir}/{name}")
    }
}

fn bootargs(root: Option<&(u32, String)>, boot_partition: u32) -> String {
    let root_arg = match root {
        Some((_, uuid)) => format!("root=UUID={uuid}"),
        None => format!("root=/dev/vdb{}", boot_partition + 1),
    };
    format!("{root_arg} rw rootwait TERM=vt102 console=ttyAMA0,115200n8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_highest_matching_kernel_and_initrd_suffix() {
        let names = vec![
            "initrd.img-6.1".to_string(),
            "vmlinuz-6.1".to_string(),
            "vmlinuz-6.12".to_string(),
            "initrd.img-6.12".to_string(),
        ];

        assert_eq!(
            select_versioned_pair(&names, "/boot"),
            Some((
                "/boot/vmlinuz-6.12".to_string(),
                "/boot/initrd.img-6.12".to_string()
            ))
        );
    }

    #[test]
    fn bootargs_prefers_root_uuid_over_device_name() {
        let args = bootargs(Some(&(3, "abcd".to_string())), 2);

        assert!(args.contains("root=UUID=abcd"));
        assert!(args.contains("console=ttyAMA0,115200n8"));
    }
}
