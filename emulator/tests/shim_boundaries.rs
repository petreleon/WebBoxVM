use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn compatibility_shims_are_deprecated_with_replacement_paths() {
    let root = workspace_root();
    let lib = read(&root, "emulator/src/lib.rs");
    let loader = read(&root, "emulator/src/loader/mod.rs");

    assert!(lib.contains("use emulator::arch::arm64 instead"));
    assert!(lib.contains("use emulator::platform::virt::SystemBus instead"));
    assert!(lib.contains("use emulator::host::wasm instead"));
    assert!(loader.contains("use crate::images::iso instead"));
}

#[test]
fn maintained_code_avoids_compatibility_shims() {
    let root = workspace_root();
    let mut failures = Vec::new();
    visit_rs(&root.join("emulator/src"), &root, &mut failures);
    visit_rs(&root.join("emulator/examples"), &root, &mut failures);

    assert!(
        failures.is_empty(),
        "compatibility shim usage remains:\n{}",
        failures.join("\n")
    );
}

fn visit_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("source directory should be readable") {
        let entry = entry.expect("source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, root, failures);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            check_rs(&path, root, failures);
        }
    }
}

fn check_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    if is_compatibility_shim(path) {
        return;
    }

    let text = fs::read_to_string(path).expect("source file should be readable");
    for needle in [
        "crate::arm64",
        "emulator::arm64",
        "crate::bus",
        "emulator::bus",
        "crate::loader::iso",
        "emulator::loader::iso",
        "crate::wasm_main",
        "emulator::wasm_main",
    ] {
        if text.contains(needle) {
            failures.push(format!("{} contains `{needle}`", rel(path, root).display()));
        }
    }
}

fn is_compatibility_shim(path: &Path) -> bool {
    path.ends_with("emulator/src/arm64.rs")
        || path.ends_with("emulator/src/bus.rs")
        || path.ends_with("emulator/src/loader/iso.rs")
        || path.ends_with("emulator/src/loader/mod.rs")
        || path.ends_with("emulator/src/wasm_main.rs")
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
