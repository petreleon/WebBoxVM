use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn runtime_owns_machine_orchestration() {
    let root = workspace_root();
    let runtime = read(&root, "emulator/src/runtime.rs");
    let arm64_machine = read(&root, "emulator/src/arch/arm64/machine.rs");

    assert!(runtime.contains("pub struct Machine"));
    assert!(runtime.contains("mod run;"));
    assert!(!arm64_machine.contains("pub struct Machine"));
    assert!(arm64_machine.contains("crate::runtime::Machine"));
}

#[test]
fn arch_arm64_owns_cpu_implementation() {
    let root = workspace_root();
    let shim = read(&root, "emulator/src/arm64.rs");
    let arch = read(&root, "emulator/src/arch/arm64/mod.rs");

    assert!(arch.contains("pub struct Armv8Cpu"));
    assert!(arch.contains("mod decode;"));
    assert!(!shim.contains("pub struct Armv8Cpu"));
    assert!(shim.contains("crate::arch::arm64"));
    assert!(!root.join("emulator/src/arm64").exists());
}

#[test]
fn platform_virt_owns_system_bus_routing() {
    let root = workspace_root();
    let bus = read(&root, "emulator/src/bus.rs");
    let virt = read(&root, "emulator/src/platform/virt.rs");

    assert!(virt.contains("pub struct SystemBus"));
    assert!(virt.contains("pub mem: PhysicalMemory"));
    assert!(!bus.contains("pub struct SystemBus"));
    assert!(bus.contains("crate::platform::virt::SystemBus"));
}

#[test]
fn boot_produces_plan_runtime_owns_live_context() {
    let root = workspace_root();
    let plan = read(&root, "emulator/src/boot/plan.rs");
    let context = read(&root, "emulator/src/runtime/boot_context.rs");
    let mut failures = Vec::new();

    assert!(plan.contains("pub struct BootPlan"));
    assert!(context.contains("pub struct BootContext"));
    assert!(context.contains("BootPlan"));
    visit_boot_rs(&root.join("emulator/src/boot"), &root, &mut failures);

    assert!(
        failures.is_empty(),
        "boot owns live runtime state:\n{}",
        failures.join("\n")
    );
}

#[test]
fn host_wasm_owns_javascript_adapter() {
    let root = workspace_root();
    let shim = read(&root, "emulator/src/wasm_main.rs");
    let host = read(&root, "emulator/src/host/wasm.rs");

    assert!(host.contains("pub struct Emulator"));
    assert!(host.contains("pub(in crate::host::wasm) struct JitPendingStore"));
    assert!(!shim.contains("pub struct Emulator"));
    assert!(shim.contains("crate::host::wasm"));
}

#[test]
fn internal_code_imports_machine_from_runtime() {
    let root = workspace_root();
    let mut failures = Vec::new();
    visit_rs(&root.join("emulator/src"), &root, &mut failures);

    assert!(
        failures.is_empty(),
        "architecture boundary leak:\n{}",
        failures.join("\n")
    );
}

fn visit_boot_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("boot source directory should be readable") {
        let entry = entry.expect("boot source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_boot_rs(&path, root, failures);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            check_boot_rs(&path, root, failures);
        }
    }
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

fn check_boot_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    let text = fs::read_to_string(path).expect("boot source file should be readable");
    for needle in ["pub struct BootContext", "crate::runtime::Machine"] {
        if text.contains(needle) {
            failures.push(format!("{} contains `{needle}`", rel(path, root).display()));
        }
    }
}

fn check_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    if path.ends_with("emulator/src/arch/arm64/machine.rs")
        || path.ends_with("emulator/src/arm64.rs")
        || path.ends_with("emulator/src/bus.rs")
    {
        return;
    }
    let text = fs::read_to_string(path).expect("source file should be readable");
    for needle in [
        "crate::arm64::",
        "crate::arch::arm64::machine::Machine",
        "use crate::arch::arm64::Machine",
        "pub(in crate::arch::arm64::machine)",
        "crate::bus::SystemBus",
        "crate::wasm_main::Emulator",
    ] {
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
