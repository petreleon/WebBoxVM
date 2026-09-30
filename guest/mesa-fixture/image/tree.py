"""Confined guest namespaces, including merged-usr and dynamic-loader symlinks."""

import hashlib
import os
from pathlib import Path, PurePosixPath
import stat


def add(nodes, name, mode, data=b"", **attributes):
    parts = PurePosixPath(name).parts
    if (not parts or name != PurePosixPath(name).as_posix() or name.startswith("/")
            or any(part in (".", "..") for part in parts) or "\0" in name):
        raise ValueError("fixture member name is unsafe")
    node = {"mode": mode, "data": data, **attributes}
    if name in nodes and nodes[name] != node:
        raise ValueError(f"fixture inputs conflict at {name}")
    nodes[name] = node


def collect(root):
    if root.is_symlink() or not root.is_dir():
        raise ValueError("fixture input must be a regular directory")
    nodes = {}
    for path in sorted(root.rglob("*")):
        name = path.relative_to(root).as_posix()
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            data = os.readlink(path).encode("utf-8")
        elif stat.S_ISREG(mode):
            data = path.read_bytes()
        elif stat.S_ISDIR(mode):
            data = b""
        else:
            raise ValueError(f"unsupported input member type at {name}")
        add(nodes, name, stat.S_IFMT(mode) | stat.S_IMODE(mode), data)
    return nodes


def resolve(nodes, name):
    pending, stack, visited = list(PurePosixPath(name).parts), [], set()
    while pending:
        part = pending.pop(0)
        if part in ("", ".", "/"):
            continue
        if part == "..":
            if not stack:
                raise ValueError("fixture symlink escapes the guest root")
            stack.pop()
            continue
        candidate = "/".join([*stack, part])
        node = nodes.get(candidate)
        if node is None:
            raise ValueError(f"fixture link or required path is missing: {candidate}")
        if stat.S_ISLNK(node["mode"]):
            if candidate in visited:
                raise ValueError("fixture symlink cycle")
            visited.add(candidate)
            target = node["data"].decode("utf-8")
            if not target or "\0" in target:
                raise ValueError("empty or malformed fixture symlink")
            if target.startswith("/"):
                stack = []
            pending = target.split("/") + pending
        else:
            if pending and not stat.S_ISDIR(node["mode"]):
                raise ValueError("fixture path descends through a non-directory")
            stack.append(part)
    return "/".join(stack)


def validate(nodes):
    for name, node in nodes.items():
        for parent in PurePosixPath(name).parents:
            if str(parent) == ".":
                continue
            ancestor = nodes.get(str(parent))
            if ancestor is None or not stat.S_ISDIR(ancestor["mode"]):
                raise ValueError(f"stored fixture member has an unsafe ancestor: {name}")
        if stat.S_ISLNK(node["mode"]):
            resolve(nodes, name)


def records(nodes):
    return [{"path": name, "mode": node["mode"], "bytes": len(node["data"]),
             "sha256": hashlib.sha256(node["data"]).hexdigest(),
             **({"major": node["major"], "minor": node["minor"]} if "major" in node else {})}
            for name, node in sorted(nodes.items())]
