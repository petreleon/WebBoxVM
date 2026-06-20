use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn observability_owns_trace_state_and_helpers() {
    let root = workspace_root();
    let execute = read(&root, "emulator/src/arch/arm64/execute/mod.rs");
    let exceptions = read(
        &root,
        "emulator/src/arch/arm64/execute/system/exceptions.rs",
    );
    let runtime = read(&root, "emulator/src/runtime.rs");
    let observability = read(&root, "emulator/src/observability.rs");

    assert!(observability.contains("mod debug_dump;"));
    assert!(observability.contains("mod trace_state;"));
    assert!(observability.contains("TraceOptions"));
    assert!(observability.contains("TraceState"));
    assert!(exceptions.contains("dump_breakpoint_context"));
    assert!(runtime.contains("crate::observability"));
    assert!(!execute.contains("mod debug;"));
    assert!(!runtime.contains("mod trace_state;"));
    assert!(
        !root
            .join("emulator/src/arch/arm64/execute/debug.rs")
            .exists()
    );
    assert!(!root.join("emulator/src/runtime/trace_state.rs").exists());
    assert!(
        root.join("emulator/src/observability/debug_dump.rs")
            .exists()
    );
    assert!(
        root.join("emulator/src/observability/trace_state.rs")
            .exists()
    );
}

#[test]
fn observability_does_not_depend_on_runtime() {
    let root = workspace_root();
    let mut failures = Vec::new();
    visit_rs(
        &root.join("emulator/src/observability"),
        &root,
        &mut failures,
    );

    assert!(
        failures.is_empty(),
        "observability boundary leak:\n{}",
        failures.join("\n")
    );
}

#[test]
fn runtime_keeps_only_trace_hook_dispatch() {
    let root = workspace_root();
    let runtime = read(&root, "emulator/src/runtime.rs");

    for module in [
        "mod trace_filters;",
        "mod trace_hotspots;",
        "mod trace_memory;",
        "mod trace_paths;",
        "mod trace_stack;",
        "mod trace_syscalls;",
        "mod trace_syscalls_exec;",
        "mod trace_syscalls_write;",
    ] {
        assert!(!runtime.contains(module), "runtime still declares {module}");
    }
}

fn visit_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("observability directory should be readable") {
        let entry = entry.expect("observability directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, root, failures);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            check_rs(&path, root, failures);
        }
    }
}

fn check_rs(path: &Path, root: &Path, failures: &mut Vec<String>) {
    let text = fs::read_to_string(path).expect("observability source file should be readable");
    for needle in ["crate::runtime", "super::Machine", "BootContext"] {
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
