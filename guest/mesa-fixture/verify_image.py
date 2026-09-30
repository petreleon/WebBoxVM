"""Verify the published fixture manifest before running a real guest."""

import hashlib
import json
from pathlib import Path


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verify(directory):
    if directory.is_symlink() or not directory.is_dir():
        raise ValueError("image directory must be a regular directory")
    for name in ("Image", "initrd.cpio", "manifest.json"):
        path = directory / name
        if path.is_symlink() or not path.is_file():
            raise ValueError("missing or redirected image artifact: " + name)
    manifest = json.loads((directory / "manifest.json").read_text())
    if (manifest.get("schema") != 1 or manifest.get("kind") != "webboxvm-stock-mesa-initramfs"
            or manifest.get("software_fallback_allowed") is not False):
        raise ValueError("image manifest has an unexpected contract")
    body = {key: value for key, value in manifest.items() if key != "manifest_sha256"}
    if manifest.get("manifest_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        raise ValueError("image manifest self hash differs")
    for name, field in (("Image", "kernel"), ("initrd.cpio", "initrd")):
        path = directory / name
        if manifest[field] != {"bytes": path.stat().st_size, "sha256": digest(path)}:
            raise ValueError("image bytes differ from manifest: " + name)
    return manifest
