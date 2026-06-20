use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn vm_handle_is_backed_by_runtime_machine() {
    let root = workspace_root();
    let vm = read(&root, "emulator/src/api/vm.rs");

    assert!(vm.contains("crate::runtime::Machine"));
    assert!(vm.contains("machine: Machine"));
    assert!(vm.contains("Machine::new"));
    assert!(!vm.contains("pub machine"));
}

#[test]
fn public_api_does_not_import_arch_or_platform_internals() {
    let root = workspace_root();
    let mut failures = Vec::new();
    visit_rs(&root.join("emulator/src/api"), &root, &mut failures);

    assert!(
        failures.is_empty(),
        "api boundary leak:\n{}",
        failures.join("\n")
    );
}

fn visit_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("api source directory should be readable") {
        let entry = entry.expect("api source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, root, failures);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            check_rs(&path, root, failures);
        }
    }
}

fn check_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    let text = fs::read_to_string(path).expect("api source file should be readable");
    for needle in ["crate::arch", "crate::platform", "Armv8Cpu", "SystemBus"] {
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
