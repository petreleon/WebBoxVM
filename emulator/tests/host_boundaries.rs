use std::fs;
use std::path::{Path, PathBuf};

const NATIVE_BOOT_EXAMPLES: &[&str] = &[
    "emulator/examples/terminal.rs",
    "emulator/examples/iso_info.rs",
    "emulator/examples/wait_uart.rs",
    "emulator/examples/boot_test_app/mod.rs",
    "emulator/examples/boot_test_app/debug.rs",
    "emulator/examples/boot_test_app/kernel.rs",
    "emulator/examples/boot_test_app/prompt_script.rs",
    "emulator/examples/boot_test_app/util.rs",
];

#[test]
fn host_native_owns_cli_boot_construction() {
    let root = workspace_root();
    let native = read(&root, "emulator/src/host/native.rs");

    assert!(native.contains("pub type NativeVm"));
    assert!(native.contains("pub fn boot_from_image"));
    assert!(native.contains("crate::images::iso::load_iso_boot_image"));
}

#[test]
fn native_boot_examples_use_host_adapter() {
    let root = workspace_root();
    let mut failures = Vec::new();

    for path in NATIVE_BOOT_EXAMPLES {
        check_example(&root.join(path), &root, &mut failures);
    }

    assert!(
        failures.is_empty(),
        "native boot example bypasses host adapter:\n{}",
        failures.join("\n")
    );
}

fn check_example(path: &Path, root: &Path, failures: &mut Vec<String>) {
    let text = fs::read_to_string(path).expect("example should be readable");
    for needle in ["emulator::boot::BootContext", "emulator::loader::iso"] {
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
