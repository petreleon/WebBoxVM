#[path = "source_file_limits/support.rs"]
mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use support::{
    collect_failures, is_source_file, physical_line_count, workspace_root, MAX_LINES, ROOT_DIRS,
};

static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn maintained_source_files_stay_under_180_lines() {
    let failures = collect_failures(&workspace_root());
    assert!(
        failures.is_empty(),
        "source files over {MAX_LINES} lines:\n{}",
        failures.join("\n")
    );
}

#[test]
fn configured_roots_exist_and_include_graphics_inputs() {
    let workspace = workspace_root();
    for root in ROOT_DIRS {
        assert!(workspace.join(root).is_dir(), "missing tracked root {root}");
    }
    for path in [
        "guest/demo.c",
        "guest/demo.h",
        "guest/link.ld",
        "guest/Makefile",
    ] {
        assert!(
            is_source_file(Path::new(path)),
            "missing source classification for {path}"
        );
    }
}

#[test]
fn rejects_a_181_line_fixture_and_accepts_180_lines() {
    let workspace = TemporaryWorkspace::new();
    write_lines(&workspace.path, "todo/allowed.md", MAX_LINES);
    write_lines(&workspace.path, "todo/rejected.md", MAX_LINES + 1);
    assert_eq!(
        collect_failures(&workspace.path),
        vec!["todo/rejected.md: 181"]
    );
}

#[test]
fn checks_nested_c_h_ld_and_makefiles() {
    let workspace = TemporaryWorkspace::new();
    for path in [
        "guest/rejected.c",
        "guest/rejected.h",
        "guest/rejected.ld",
        "guest/Makefile",
    ] {
        write_lines(&workspace.path, path, MAX_LINES + 1);
    }
    assert_eq!(
        collect_failures(&workspace.path),
        vec![
            "guest/Makefile: 181",
            "guest/rejected.c: 181",
            "guest/rejected.h: 181",
            "guest/rejected.ld: 181",
        ]
    );
}

#[test]
fn allows_only_the_reviewed_legal_and_patch_exemptions() {
    let workspace = TemporaryWorkspace::new();
    write_lines(&workspace.path, "LICENSE.md", MAX_LINES + 1);
    write_lines(
        &workspace.path,
        "patches/wasm-bindgen-memory64-threads.patch",
        MAX_LINES + 1,
    );
    write_lines(&workspace.path, "guest/LICENSE.md", MAX_LINES + 1);
    write_lines(&workspace.path, "patches/other.patch", MAX_LINES + 1);
    assert_eq!(
        collect_failures(&workspace.path),
        vec!["guest/LICENSE.md: 181", "patches/other.patch: 181",]
    );
}

#[test]
fn counts_physical_lines_at_the_boundary() {
    assert_eq!(physical_line_count(&"line\n".repeat(MAX_LINES)), MAX_LINES);
    assert_eq!(
        physical_line_count(&"line\n".repeat(MAX_LINES + 1)),
        MAX_LINES + 1
    );
    assert_eq!(physical_line_count("first\nsecond"), 2);
}

struct TemporaryWorkspace {
    path: PathBuf,
}

impl TemporaryWorkspace {
    fn new() -> Self {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "webboxvm-source-limit-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temporary workspace should be created");
        Self { path }
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_lines(workspace: &Path, relative_path: &str, lines: usize) {
    let path = workspace.join(relative_path);
    fs::create_dir_all(path.parent().expect("fixture should have a parent")).unwrap();
    fs::write(path, "line\n".repeat(lines)).unwrap();
}
