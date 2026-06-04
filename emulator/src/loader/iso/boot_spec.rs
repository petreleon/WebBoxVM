use super::bootargs::DEFAULT_ISO_BOOTARGS;
use super::filesystem::IsoFs;
use super::grub::{BootSpec, parse_grub_boot_spec};

const GRUB_CONFIG_CANDIDATES: &[&str] = &[
    "/boot/grub/grub.cfg",
    "/efi/boot/grub.cfg",
    "/grub/grub.cfg",
];

const FALLBACK_PAIRS: &[(&str, &str)] = &[
    ("/casper/vmlinuz", "/casper/initrd"),
    ("/casper/vmlinuz", "/casper/initrd.gz"),
    ("/casper/vmlinuz.efi", "/casper/initrd"),
    ("/install.a64/vmlinuz", "/install.a64/initrd.gz"),
    ("/install/vmlinuz", "/install/initrd.gz"),
    ("/images/pxeboot/vmlinuz", "/images/pxeboot/initrd.img"),
    ("/boot/vmlinuz", "/boot/initrd.img"),
    ("/vmlinuz", "/initrd"),
    ("/vmlinuz", "/initrd.gz"),
    ("/image", "/initrd"),
    ("/image.gz", "/initrd.gz"),
];

pub(super) fn find_boot_spec(fs: &IsoFs<'_>) -> Result<BootSpec, String> {
    for cfg_path in GRUB_CONFIG_CANDIDATES {
        if let Ok(config) = fs.read_text_file(cfg_path)
            && let Some(spec) = parse_grub_boot_spec(&config, |path| fs.exists(path))
        {
            return Ok(spec);
        }
    }

    for (kernel_path, initrd_path) in FALLBACK_PAIRS {
        if fs.exists(kernel_path) && fs.exists(initrd_path) {
            return Ok(BootSpec {
                kernel_path: (*kernel_path).to_string(),
                initrd_paths: vec![(*initrd_path).to_string()],
                bootargs: DEFAULT_ISO_BOOTARGS.to_string(),
            });
        }
    }

    Err("no supported ARM64 Linux kernel/initrd pair found in ISO".to_string())
}
