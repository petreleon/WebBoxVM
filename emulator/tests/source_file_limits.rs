use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 180;
const SOURCE_EXTENSIONS: &[&str] = &["css", "html", "js", "json", "md", "mjs", "rs", "sh", "toml"];
const ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "Makefile",
    "README.md",
    "future.md",
    "models.md",
    "test_wasm64.mjs",
    "todo.md",
    "vision.md",
    "wasm64-wasip1.json",
];
const ROOT_DIRS: &[&str] = &["emulator", "models", "scripts", "web"];

#[test]
fn maintained_source_files_stay_under_180_lines() {
    let workspace = workspace_root();
    let mut failures = Vec::new();

    for file in ROOT_FILES {
        check_file(&workspace.join(file), &workspace, &mut failures);
    }
    for dir in ROOT_DIRS {
        visit_dir(&workspace.join(dir), &workspace, &mut failures);
    }

    assert!(
        failures.is_empty(),
        "source files over {MAX_LINES} lines:\n{}",
        failures.join("\n")
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("emulator crate should live under workspace root")
        .to_path_buf()
}

fn visit_dir(path: &Path, workspace: &Path, failures: &mut Vec<String>) {
    if is_exempt_dir(path) {
        return;
    }

    let entries = fs::read_dir(path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    });
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, workspace, failures);
        } else {
            check_file(&path, workspace, failures);
        }
    }
}

fn check_file(path: &Path, workspace: &Path, failures: &mut Vec<String>) {
    if !is_source_file(path) {
        return;
    }

    let contents = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    });
    let lines = contents.lines().count();
    if lines > MAX_LINES {
        failures.push(format!("{}: {lines}", relative(path, workspace).display()));
    }
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
}

fn is_exempt_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(name, "pkg" | "target")
}

fn relative<'a>(path: &'a Path, workspace: &Path) -> &'a Path {
    path.strip_prefix(workspace).unwrap_or(path)
}
