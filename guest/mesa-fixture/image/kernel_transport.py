"""Transfer validated module bytes to private tmpfs; return only depmod metadata."""

import io
from pathlib import Path
import tarfile

from kernel_archive import MAX_TAR, member_name


def module_tar(root, release):
    source = root / "usr/lib/modules" / release
    names = {"usr", "usr/lib", "usr/lib/modules", source.relative_to(root).as_posix(), "lib", "lib/modules"}
    names.update(path.relative_to(root).as_posix() for path in source.rglob("*"))
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:", format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted(names):
            member_name(name)
            path, member = root / name, tarfile.TarInfo(name)
            member.uid = member.gid = member.mtime = 0
            if path.is_symlink():
                if name != "lib/modules" or str(path.readlink()) != "../usr/lib/modules":
                    raise ValueError("unexpected module transport symlink")
                member.type, member.linkname, member.mode = tarfile.SYMTYPE, "../usr/lib/modules", 0o777
                archive.addfile(member)
            elif path.is_dir():
                member.type, member.mode = tarfile.DIRTYPE, 0o755
                archive.addfile(member)
            elif path.is_file():
                raw = path.read_bytes()
                member.size, member.mode = len(raw), 0o644
                archive.addfile(member, io.BytesIO(raw))
            else:
                raise ValueError("unexpected module transport member")
            if output.tell() > MAX_TAR:
                raise ValueError("module transport exceeds its byte bound")
    raw = output.getvalue()
    if len(raw) > MAX_TAR:
        raise ValueError("module transport exceeds its byte bound")
    return raw


def metadata(raw, root, release):
    if len(raw) > 16 * 1024 * 1024:
        raise ValueError("depmod metadata exceeds its byte bound")
    prefix, pending = "usr/lib/modules/" + release + "/", {}
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
        for member in archive.getmembers():
            name = member_name(member.name)
            tail = name.removeprefix(prefix)
            if (not name.startswith(prefix) or "/" in tail or not tail.startswith("modules.")
                    or name in pending or not member.isfile()):
                raise ValueError("depmod returned unsafe or duplicate metadata")
            pending[name] = archive.extractfile(member).read()
    for required in ("modules.dep", "modules.dep.bin", "modules.softdep"):
        if prefix + required not in pending:
            raise ValueError("depmod returned incomplete metadata")
    for name, payload in pending.items():
        destination = root / name
        if destination.is_symlink():
            raise ValueError("depmod metadata destination is a symlink")
        destination.write_bytes(payload)
        destination.chmod(0o644)
