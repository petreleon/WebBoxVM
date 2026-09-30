#!/usr/bin/env python3
"""Authenticate exported runtime bytes and their matching installed Mesa inputs."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat

from image import tree
from verify_image import digest


def records(nodes):
    rows = []
    for name, node in sorted(nodes.items()):
        if stat.S_ISDIR(node["mode"]):
            continue
        row = {"path": "/" + name}
        if stat.S_ISLNK(node["mode"]):
            row["symlink"] = node["data"].decode()
        elif stat.S_ISREG(node["mode"]):
            row.update(bytes=len(node["data"]), sha256=hashlib.sha256(node["data"]).hexdigest())
        else:
            raise ValueError("unexpected runtime node")
        rows.append(row)
    return rows


def verify(build):
    manifest_path = build / "manifests/runtime.json"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ValueError("runtime manifest must be a regular file")
    manifest = json.loads(manifest_path.read_text())
    nodes = tree.collect(build / "runtime-rootfs")
    tree.validate(nodes)
    for node in nodes.values():
        if stat.S_ISDIR(node["mode"]) and node["mode"] & 0o111 != 0o111:
            raise ValueError("runtime directory lacks search permissions")
    for name in manifest["tools"].values():
        node = nodes[tree.resolve(nodes, name.lstrip("/"))]
        if not stat.S_ISREG(node["mode"]) or not node["mode"] & 0o111:
            raise ValueError("runtime tool is not executable: " + name)
    expected = manifest["files"]
    if (manifest.get("schema") != 1 or manifest.get("drivers") != ["virgl"]
            or manifest.get("legacy_dri_aliases") != []
            or manifest.get("icds") != ["opt/mesa-f02/share/vulkan/icd.d/virtio_icd.aarch64.json"]):
        raise ValueError("runtime driver contract differs")
    for row in expected:
        name = row["path"]
        if name != "/" + PurePosixPath(name).relative_to("/").as_posix() or ".." in PurePosixPath(name).parts:
            raise ValueError("unsafe runtime manifest path")
    if records(nodes) != sorted(expected, key=lambda row: row["path"]):
        raise ValueError("runtime bytes or links differ from the compiled manifest")
    if sum(len(node["data"]) for node in nodes.values() if stat.S_ISREG(node["mode"])) != manifest["bytes"]:
        raise ValueError("runtime byte total differs")
    installed = tree.collect(build / "mesa-destdir")
    for name, node in nodes.items():
        if name.startswith("opt/mesa-f02/") and not stat.S_ISDIR(node["mode"]):
            if installed.get(name) != node:
                raise ValueError("installed Mesa input differs from verified runtime: " + name)
    return {"result": "PASS", "scope": "runtime bytes/links and matching installed Mesa files",
            "files": len(expected), "bytes": manifest["bytes"], "runtime_manifest_sha256": digest(manifest_path)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(verify(args.build)))
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        parser.exit(2, f"FAIL: {error}\n")


if __name__ == "__main__":
    main()
