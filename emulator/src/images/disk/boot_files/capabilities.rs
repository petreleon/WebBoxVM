use super::{Ext4, join_path};
use ext4_view::Ext4Error;
use systemd::usable_serial_getty_template;

mod systemd;

const SERIAL_GETTY_TEMPLATE_PATHS: [&str; 2] = [
    "/usr/lib/systemd/system/serial-getty@.service",
    "/lib/systemd/system/serial-getty@.service",
];
const SERIAL_GETTY_VENDOR_INSTANCE_PATHS: [&str; 2] = [
    "/usr/lib/systemd/system/serial-getty@ttyAMA0.service",
    "/lib/systemd/system/serial-getty@ttyAMA0.service",
];
const SERIAL_GETTY_VENDOR_DROPIN_PATHS: [&str; 4] = [
    "/usr/lib/systemd/system/serial-getty@ttyAMA0.service.d",
    "/usr/lib/systemd/system/serial-getty@.service.d",
    "/lib/systemd/system/serial-getty@ttyAMA0.service.d",
    "/lib/systemd/system/serial-getty@.service.d",
];
const PERSISTENT_SYSTEMD_OVERRIDE_ROOTS: [&str; 4] = [
    "/etc/systemd/system.control",
    "/etc/systemd/system.attached",
    "/etc/systemd/system",
    "/usr/local/lib/systemd/system",
];
const SYSTEMD_UNIT_ROOTS: [&str; 6] = [
    "/etc/systemd/system.control",
    "/etc/systemd/system.attached",
    "/etc/systemd/system",
    "/usr/local/lib/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];
const SERIAL_GETTY_UNIT_NAMES: [&str; 2] =
    ["serial-getty@ttyAMA0.service", "serial-getty@.service"];
const SERIAL_GETTY_SHARED_DROPIN_NAMES: [&str; 2] = ["service.d", "serial-.service.d"];

pub(super) fn staged_smp_root(fs: &Ext4) -> bool {
    is_debian(fs)
        && exists(fs, "/bin/sh")
        && exists(fs, "/sbin/init")
        && exists_any(fs, &["/usr/lib/systemd/systemd", "/lib/systemd/systemd"])
        && serial_getty_is_unmodified(fs)
        && exists_any(fs, &["/usr/bin/grep", "/bin/grep"])
}

pub(super) fn kernel_cpu_hotplug(fs: &Ext4, suffix: Option<&str>) -> bool {
    let Some(suffix) = suffix.filter(|suffix| !suffix.is_empty()) else {
        return false;
    };
    let config = format!("config-{suffix}");
    ["/", "/boot"].into_iter().any(|dir| {
        let path = join_path(dir, &config);
        fs.read(path.as_str())
            .ok()
            .is_some_and(|data| has_line(&data, b"CONFIG_HOTPLUG_CPU=y"))
    })
}

fn serial_getty_is_unmodified(fs: &Ext4) -> bool {
    persistent_serial_getty_overrides_absent(fs)
        && shared_serial_getty_dropins_absent(fs)
        && SERIAL_GETTY_VENDOR_INSTANCE_PATHS
            .iter()
            .all(|path| path_is_absent(fs, path))
        && SERIAL_GETTY_VENDOR_DROPIN_PATHS
            .iter()
            .all(|path| path_is_absent(fs, path))
        && vendor_serial_getty_templates_are_known(fs)
}

fn vendor_serial_getty_templates_are_known(fs: &Ext4) -> bool {
    let mut found = false;
    for path in SERIAL_GETTY_TEMPLATE_PATHS {
        match fs.read(path) {
            Ok(data) => {
                found = true;
                if !usable_serial_getty_template(&data) {
                    return false;
                }
            }
            Err(Ext4Error::NotFound) => {}
            Err(_) => return false,
        }
    }
    found
}

fn shared_serial_getty_dropins_absent(fs: &Ext4) -> bool {
    SYSTEMD_UNIT_ROOTS.iter().all(|root| {
        SERIAL_GETTY_SHARED_DROPIN_NAMES
            .iter()
            .all(|name| path_is_absent(fs, &format!("{root}/{name}")))
    })
}

fn persistent_serial_getty_overrides_absent(fs: &Ext4) -> bool {
    PERSISTENT_SYSTEMD_OVERRIDE_ROOTS.iter().all(|root| {
        SERIAL_GETTY_UNIT_NAMES.iter().all(|unit| {
            let unit_path = format!("{root}/{unit}");
            let dropin_path = format!("{unit_path}.d");
            path_is_absent(fs, &unit_path) && path_is_absent(fs, &dropin_path)
        })
    })
}

fn path_is_absent(fs: &Ext4, path: &str) -> bool {
    matches!(fs.symlink_metadata(path), Err(Ext4Error::NotFound))
}

fn is_debian(fs: &Ext4) -> bool {
    fs.read("/etc/os-release")
        .ok()
        .is_some_and(|data| has_line(&data, b"ID=debian") || has_line(&data, b"ID=\"debian\""))
}

fn exists(fs: &Ext4, path: &str) -> bool {
    fs.exists(path).unwrap_or(false)
}

fn exists_any(fs: &Ext4, paths: &[&str]) -> bool {
    paths.iter().any(|path| exists(fs, path))
}

fn has_line(data: &[u8], expected: &[u8]) -> bool {
    data.split(|byte| *byte == b'\n')
        .any(|line| line.strip_suffix(b"\r").unwrap_or(line) == expected)
}

#[cfg(test)]
mod tests;
