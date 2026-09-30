#!/usr/bin/env python3
"""Extract the canonical ARM64 Image and GPU plus MMIO transport module closure."""

import argparse
import json
from pathlib import Path
import shutil
import tempfile

from kernel_archive import ar_members, digest, extract, package_identity
from kernel_modules import closure, copy_closure, dependencies
from kernel_tools import BUILDER, Commands, depmod, verify_builder

RELEASE = "6.12.94+deb13-arm64"
PACKAGE = "linux-image-" + RELEASE
VERSION = "6.12.94-1"
PACKAGE_BYTES = 92732600
PACKAGE_SHA256 = "72db7fcfb443a4b03448bda98f4e7c1a1fa0d6c21fc57f0b119d704442f8ad49"
IMAGE_SHA256 = "cbe59a02e7ea979a150661032440c94e2c4db0b735af2416e11ae5cac15a58e4"
VIRTIO_SHA256 = "6ad520592a3ccb4c62475563bd1ca3d4e6874de1b8e6d58433cc0baf032dd47a"
MMIO_SHA256 = "7ced5d7abb0734d0bbae457e60ccae04893bc0d82313c8c568dd1933f00b2fc5"
MODULE_ROOTS = ("virtio_mmio", "virtio_gpu")


def confined_parent(path):
    for parent in [path, *path.parents]:
        if parent.is_symlink():
            raise ValueError("output/log paths cannot traverse a symlink")
        if parent.exists() and not parent.is_dir():
            raise ValueError("output/log parent is not a directory")
    path.mkdir(parents=True, exist_ok=True)


def module_alias(root):
    (root / "lib").mkdir(mode=0o755)
    (root / "lib/modules").symlink_to("../usr/lib/modules")


def file_records(root):
    records = []
    for path in sorted(root.rglob("*")):
        name = path.relative_to(root).as_posix()
        if path.is_symlink():
            records.append({"path": name, "type": "symlink", "target": str(path.readlink())})
        elif path.is_file():
            raw = path.read_bytes()
            records.append({"path": name, "type": "file", "mode": 0o644,
                            "bytes": len(raw), "sha256": digest(raw)})
        elif not path.is_dir():
            raise ValueError("unexpected module output node")
    return records


def package_bytes(package):
    if package.is_symlink() or not package.is_file() or package.stat().st_size != PACKAGE_BYTES:
        raise ValueError("canonical kernel package must be a regular file with the pinned size")
    with package.open("rb") as source:
        raw = source.read(PACKAGE_BYTES + 1)
    if len(raw) != PACKAGE_BYTES or digest(raw) != PACKAGE_SHA256:
        raise ValueError("canonical kernel package size/hash mismatch")
    return raw


def build(package, output, logs, builder=BUILDER):
    package, output, logs = Path(package), Path(output), Path(logs)
    if package.is_symlink() or not package.is_file():
        raise ValueError("kernel package must be a regular file")
    if output.exists() or output.is_symlink():
        raise ValueError("preserve prior kernel output; choose a new destination")
    confined_parent(output.parent)
    confined_parent(logs.parent)
    raw = package_bytes(package)
    members = ar_members(raw)
    package_identity(members["control.tar.xz"], PACKAGE, VERSION)
    commands = Commands(logs)
    workspace = Path(tempfile.mkdtemp(prefix=".kernel-staging-", dir=output.parent))
    (logs / "workspace.json").write_text(json.dumps({"workspace": str(workspace)}) + "\n")
    try:
        identity = verify_builder(commands, builder)
        full, publish = workspace / "full", workspace / "publish"
        full.mkdir()
        source_hashes = extract(members["data.tar.xz"], full, RELEASE)
        module_alias(full)
        image = (full / ("boot/vmlinuz-" + RELEASE)).read_bytes()
        virtio = "usr/lib/modules/" + RELEASE + "/kernel/drivers/gpu/drm/virtio/virtio-gpu.ko.xz"
        mmio = "usr/lib/modules/" + RELEASE + "/kernel/drivers/virtio/virtio_mmio.ko.xz"
        if (len(image) < 64 or image[56:60] != b"ARM\x64" or digest(image) != IMAGE_SHA256
                or source_hashes.get(virtio) != VIRTIO_SHA256 or source_hashes.get(mmio) != MMIO_SHA256):
            raise ValueError("canonical Image or virtio module bytes differ from their pins")
        depmod(commands, builder, full, RELEASE, "03-full-depmod")
        source = full / "usr/lib/modules" / RELEASE
        selected, edges = closure(source, MODULE_ROOTS)
        destination = publish / "modules/usr/lib/modules" / RELEASE
        destination.mkdir(parents=True)
        copy_closure(source, destination, selected)
        module_alias(publish / "modules")
        depmod(commands, builder, publish / "modules", RELEASE, "04-closure-depmod")
        final_selected, final_edges = closure(destination, MODULE_ROOTS)
        if final_selected != selected or final_edges != edges:
            raise ValueError("pruned depmod metadata changed the dependency closure")
        final_hard = dependencies((destination / "modules.dep").read_text())
        if set(final_hard) != set(selected):
            raise ValueError("pruned metadata references a module outside the selected closure")
        for path in (publish / "modules").rglob("*"):
            if not path.is_symlink():
                path.chmod(0o755 if path.is_dir() else 0o644)
        (publish / "Image").write_bytes(image)
        manifest = {"schema": 1, "result": "PASS", "release": RELEASE,
            "package": {"name": PACKAGE, "version": VERSION, "architecture": "arm64",
                        "bytes": len(raw), "sha256": digest(raw)},
            "kernel": {"bytes": len(image), "sha256": digest(image)}, "builder": identity,
            "module_roots": MODULE_ROOTS, "selected_modules": selected, "dependencies": edges,
            "files": file_records(publish / "modules"), "commands": commands.rows,
            "maintainer_scripts_executed": False, "installed_disk_used": False}
        package_bytes(package)
        (publish / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
        if output.exists() or output.is_symlink():
            raise ValueError("kernel destination appeared during extraction")
        publish.rename(output)
    except Exception:
        (logs / "failure-workspace.json").write_text(json.dumps({"retained_workspace": str(workspace)}) + "\n")
        raise
    shutil.rmtree(workspace)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("package", "output", "logs"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--builder", default=BUILDER)
    args = parser.parse_args()
    try:
        result = build(args.package, args.output, args.logs, args.builder)
    except (OSError, ValueError) as error:
        raise SystemExit(str(error)) from error
    print(json.dumps({"result": result["result"], "modules": len(result["selected_modules"]),
                      "kernel": result["kernel"], "output": str(args.output)}))


if __name__ == "__main__":
    main()
