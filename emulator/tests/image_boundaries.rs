use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn images_own_iso_and_kernel_parsing() {
    let root = workspace_root();
    let iso = read(&root, "emulator/src/images/iso/mod.rs");
    let kernel = read(&root, "emulator/src/images/kernel.rs");
    let loader_iso = read(&root, "emulator/src/loader/iso.rs");

    assert!(iso.contains("pub struct IsoBootImage"));
    assert!(kernel.contains("pub fn parse_kernel_image"));
    assert!(!loader_iso.contains("pub struct IsoBootImage"));
    assert!(loader_iso.contains("crate::images::iso"));
}

#[test]
fn images_are_pure_byte_parsers() {
    let root = workspace_root();
    let mut failures = Vec::new();
    visit_rs(&root.join("emulator/src/images"), &root, &mut failures);

    assert!(
        failures.is_empty(),
        "image parser contains live VM side effects:\n{}",
        failures.join("\n")
    );
}

#[test]
fn boot_uses_images_for_iso_boot_artifacts() {
    let root = workspace_root();
    let boot_iso = read(&root, "emulator/src/boot/iso.rs");

    assert!(boot_iso.contains("crate::images::iso::load_iso_boot_image"));
    assert!(!boot_iso.contains("crate::loader::iso"));
}

fn visit_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("image source directory should be readable") {
        let entry = entry.expect("image source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, root, failures);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            check_rs(&path, root, failures);
        }
    }
}

fn check_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    let text = fs::read_to_string(path).expect("image source file should be readable");
    for needle in ["SystemBus", "setup_efi_tables", "write_bytes(", "std::fs"] {
        if text.contains(needle) {
            failures.push(format!("{} contains `{needle}`", rel(path, root).display()));
        }
    }
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("emulator crate should live under workspace root")
        .to_path_buf()
}

fn rel<'a>(path: &'a Path, root: &Path) -> &'a Path {
    path.strip_prefix(root).unwrap_or(path)
}
