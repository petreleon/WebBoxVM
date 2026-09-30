#!/usr/bin/env python3
"""Assemble a deterministic graphics initramfs without editing installed images."""

import argparse
import hashlib
import json
from pathlib import Path
import stat
import tempfile

import cpio
import tree

HERE = Path(__file__).resolve().parent
REQUIRED = ("bin/sh", "bin/stty", "bin/base64", "usr/sbin/modprobe", "usr/bin/eglinfo", "usr/bin/vulkaninfo",
            "usr/bin/webboxvm-mesa-gles", "usr/bin/webboxvm-mesa-vulkan",
            "lib/aarch64-linux-gnu/libvulkan.so.1",
            "opt/mesa-f02/lib/libEGL.so.1", "opt/mesa-f02/lib/libGLESv2.so.2",
            "opt/mesa-f02/lib/libgallium-25.3.6.so",
            "opt/mesa-f02/share/vulkan/icd.d/virtio_icd.aarch64.json",
            "usr/lib/modules/6.12.94+deb13-arm64/modules.dep",
            "usr/lib/modules/6.12.94+deb13-arm64/kernel/drivers/virtio/virtio_mmio.ko.xz",
            "usr/lib/modules/6.12.94+deb13-arm64/kernel/drivers/gpu/drm/virtio/virtio-gpu.ko.xz")
EXECUTABLE = set(REQUIRED[:8])


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def assemble(rootfs, modules, kernel, output, probes=None):
    if output.exists() or output.is_symlink():
        raise ValueError("preserve prior fixture; output must be a fresh directory")
    inputs = tree.collect(rootfs)
    module_nodes = tree.collect(modules)
    probe_nodes = tree.collect(probes) if probes is not None else {}
    if not kernel.is_file() or kernel.is_symlink():
        raise ValueError("fixture kernel must be a regular file")
    kernel_bytes = kernel.read_bytes()
    if len(kernel_bytes) < 64 or kernel_bytes[56:60] != b"ARM\x64":
        raise ValueError("fixture kernel must be an uncompressed ARM64 Linux Image")
    nodes = dict(inputs)
    for collection in (module_nodes, probe_nodes):
        for name, node in collection.items():
            tree.add(nodes, name, node["mode"], node["data"])
    for name, mode in (("dev", 0o755), ("proc", 0o755), ("sys", 0o755),
                       ("tmp", 0o1777), ("run", 0o700)):
        if name not in nodes:
            tree.add(nodes, name, stat.S_IFDIR | mode)
    tree.add(nodes, "dev/console", stat.S_IFCHR | 0o600, major=5, minor=1)
    tree.add(nodes, "dev/null", stat.S_IFCHR | 0o666, major=1, minor=3)
    tree.add(nodes, "dev/kmsg", stat.S_IFCHR | 0o600, major=1, minor=11)
    tree.add(nodes, "init", stat.S_IFREG | 0o755, (HERE / "init.sh").read_bytes())
    tree.validate(nodes)
    for name in REQUIRED:
        node = nodes[tree.resolve(nodes, name)]
        if not stat.S_ISREG(node["mode"]) or not node["data"]:
            raise ValueError(f"required driver/tool is not a nonempty file: {name}")
        if name in EXECUTABLE and not node["mode"] & 0o111:
            raise ValueError(f"required tool is not executable: {name}")
    manifest = {"schema": 1, "kind": "webboxvm-stock-mesa-initramfs",
                "kernel": {"bytes": len(kernel_bytes), "sha256": hashlib.sha256(kernel_bytes).hexdigest()},
                "rootfs_sha256": hashlib.sha256(canonical(tree.records(inputs))).hexdigest(),
                "modules_sha256": hashlib.sha256(canonical(tree.records(module_nodes))).hexdigest(),
                "probes_sha256": hashlib.sha256(canonical(tree.records(probe_nodes))).hexdigest(),
                "files": tree.records(nodes), "software_fallback_allowed": False}
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        staging = Path(temporary)
        manifest["initrd"] = cpio.write(staging / "initrd.cpio", nodes)
        (staging / "Image").write_bytes(kernel_bytes)
        if (inputs != tree.collect(rootfs) or module_nodes != tree.collect(modules)
                or (probes is not None and probe_nodes != tree.collect(probes)) or kernel_bytes != kernel.read_bytes()):
            raise ValueError("fixture inputs changed during assembly")
        manifest["manifest_sha256"] = hashlib.sha256(canonical(manifest)).hexdigest()
        (staging / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        if output.exists() or output.is_symlink():
            raise ValueError("fixture destination appeared during assembly")
        staging.rename(output)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("rootfs", "modules", "kernel", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--probes", required=True, type=Path)
    args = parser.parse_args()
    try:
        result = assemble(args.rootfs, args.modules, args.kernel, args.output, args.probes)
    except (ValueError, OSError, UnicodeError) as error:
        parser.exit(2, f"FAIL: {error}\n")
    print(json.dumps({"result": "PASS", "scope": "image assembly only", "files": len(result["files"]),
                      "kernel": result["kernel"], "initrd": result["initrd"],
                      "manifest_sha256": result["manifest_sha256"]}, indent=2))


if __name__ == "__main__":
    main()
