use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_LINES: usize = 180;
const SOURCE_EXTENSIONS: &[&str] = &[
    "c", "css", "h", "html", "js", "json", "ld", "md", "mjs", "patch", "py", "rs", "sh", "toml",
];
const ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "LICENSE.md",
    "Makefile",
    "README.md",
    "future.md",
    "models.md",
    "test_wasm64.mjs",
    "todo.md",
    "vision.md",
    "wasm64-wasip1.json",
];
pub const ROOT_DIRS: &[&str] = &[
    "emulator", "models", "scripts", "web", "todo", "guest", "research", "patches",
];
// Exact audit list; provenance and hashes live in the F06.2 exemption ledger.
const EXEMPT_FILES: &[&str] = &["LICENSE.md", "patches/wasm-bindgen-memory64-threads.patch"];
const GENERATED_OUTPUT_DIRS: &[&str] = &["web/pkg", "web/pkg-threaded"];

pub fn collect_failures(workspace: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for file in ROOT_FILES {
        check_file(&workspace.join(file), workspace, &mut failures);
    }
    for dir in ROOT_DIRS {
        visit_dir(&workspace.join(dir), workspace, &mut failures);
    }
    failures.sort();
    failures
}

fn visit_dir(path: &Path, workspace: &Path, failures: &mut Vec<String>) {
    if !path.is_dir() || is_generated_output_dir(path, workspace) {
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .map(|entry| entry.expect("directory entry should be readable").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            visit_dir(&path, workspace, failures);
        } else {
            check_file(&path, workspace, failures);
        }
    }
}

fn check_file(path: &Path, workspace: &Path, failures: &mut Vec<String>) {
    if !path.is_file() || is_exempt_file(path, workspace) || !is_source_file(path) {
        return;
    }
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let lines = physical_line_count(&contents);
    if lines > MAX_LINES {
        failures.push(format!("{}: {lines}", relative(path, workspace).display()));
    }
}

pub fn is_source_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("Makefile")
        || path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
}

fn is_exempt_file(path: &Path, workspace: &Path) -> bool {
    let path = relative(path, workspace);
    EXEMPT_FILES.iter().any(|exempt| path == Path::new(exempt))
}

fn is_generated_output_dir(path: &Path, workspace: &Path) -> bool {
    let path = relative(path, workspace).to_string_lossy();
    GENERATED_OUTPUT_DIRS.iter().any(|output| path == *output)
}

pub fn physical_line_count(contents: &str) -> usize {
    contents.lines().count()
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("emulator crate should live under workspace root")
        .to_path_buf()
}

fn relative<'a>(path: &'a Path, workspace: &Path) -> &'a Path {
    path.strip_prefix(workspace).unwrap_or(path)
}
