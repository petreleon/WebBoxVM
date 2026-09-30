"""Read a pinned .deb as data; never install it or execute maintainer scripts."""

import hashlib
import io
import lzma
from pathlib import Path
import tarfile

MAX_TAR = 256 * 1024 * 1024


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def ar_members(raw):
    if raw[:8] != b"!<arch>\n":
        raise ValueError("kernel package is not a Debian ar archive")
    members, offset = {}, 8
    while offset < len(raw):
        header = raw[offset:offset + 60]
        if len(header) != 60 or header[58:] != b"`\n":
            raise ValueError("malformed ar header")
        name = header[:16].decode("ascii").strip().rstrip("/")
        if name not in {"debian-binary", "control.tar.xz", "data.tar.xz"} or name in members:
            raise ValueError("unexpected or duplicate Debian ar member")
        size_text = header[48:58].strip()
        if not size_text.isdigit():
            raise ValueError("malformed ar size")
        size = int(size_text)
        start, end = offset + 60, offset + 60 + size
        if end > len(raw) or (size % 2 and raw[end:end + 1] != b"\n"):
            raise ValueError("truncated ar member or invalid padding")
        members[name] = raw[start:end]
        offset = end + size % 2
    if set(members) != {"debian-binary", "control.tar.xz", "data.tar.xz"} or members["debian-binary"] != b"2.0\n":
        raise ValueError("kernel package has an unexpected Debian format")
    return members


def member_name(raw):
    if raw.startswith("./"):
        raw = raw[2:]
    raw = raw.rstrip("/")
    if raw in {"", "."}:
        return ""
    if (raw.startswith("/") or "\\" in raw or any(ord(c) < 32 or ord(c) == 127 for c in raw)
            or any(part in {"", ".", ".."} for part in raw.split("/"))):
        raise ValueError("unsafe kernel archive member path")
    return raw


def open_tar(raw, limit=MAX_TAR):
    with lzma.LZMAFile(io.BytesIO(raw)) as source:
        unpacked = source.read(limit + 1)
    if len(unpacked) > limit:
        raise ValueError("kernel tar expansion exceeds its bound")
    archive = tarfile.open(fileobj=io.BytesIO(unpacked), mode="r:")
    nodes = {}
    for member in archive.getmembers():
        name = member_name(member.name)
        if not name:
            if not member.isdir():
                raise ValueError("kernel archive root is not a directory")
            continue
        if name in nodes or not (member.isdir() or member.isfile()):
            raise ValueError("duplicate or special kernel archive member")
        nodes[name] = member
    if len(nodes) > 20_000:
        raise ValueError("too many kernel archive members")
    for name in nodes:
        parent = Path(name).parent
        while str(parent) != ".":
            node = nodes.get(parent.as_posix())
            if node is None or not node.isdir():
                raise ValueError("kernel member has a missing/non-directory ancestor")
            parent = parent.parent
    return archive, nodes


def package_identity(raw, package, version):
    archive, nodes = open_tar(raw, 4 * 1024 * 1024)
    with archive:
        control = nodes.get("control")
        if control is None or not control.isfile():
            raise ValueError("kernel package lacks a regular control file")
        fields = {}
        for line in archive.extractfile(control).read().decode("utf-8").splitlines():
            if not line or line[0].isspace():
                continue
            key, separator, value = line.partition(":")
            if not separator or key in fields:
                raise ValueError("malformed Debian control fields")
            fields[key] = value.strip()
        for key, expected in {"Package": package, "Version": version, "Architecture": "arm64"}.items():
            if fields.get(key) != expected:
                raise ValueError("kernel package identity mismatch: " + key)


def extract(raw, destination, release):
    archive, nodes = open_tar(raw)
    prefix = "usr/lib/modules/" + release + "/"
    image = "boot/vmlinuz-" + release
    hashes = {}
    with archive:
        for name, member in nodes.items():
            if not member.isfile() or not (name == image or name.startswith(prefix)):
                continue
            payload = archive.extractfile(member).read()
            if len(payload) != member.size:
                raise ValueError("truncated kernel file")
            path = destination / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            path.chmod(0o644)
            hashes[name] = digest(payload)
    if image not in hashes:
        raise ValueError("canonical kernel Image is missing")
    return hashes
