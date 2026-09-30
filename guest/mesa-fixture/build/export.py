"""Export guest links without asking host mount backends to timestamp targets."""
import os
import pathlib
import shutil
import hashlib
import stat


def export_tree(source, destination):
    source, destination = pathlib.Path(source), pathlib.Path(destination)
    destination.mkdir()
    for path in sorted(source.rglob('*'), key=lambda item: (len(item.parts), str(item))):
        target = destination / path.relative_to(source)
        if path.is_symlink():
            os.symlink(os.readlink(path), target)
        elif path.is_dir():
            target.mkdir()
        else:
            shutil.copy2(path, target)


def preserve_install_modes(source, destination):
    """Preserve sealed install permissions after a host-mounted recovery export."""
    source, destination = pathlib.Path(source), pathlib.Path(destination)
    for path in sorted(source.rglob('*')):
        if path.is_symlink():
            continue
        target = destination / path.relative_to(source)
        if target.is_symlink() or not target.exists():
            raise ValueError('recovered install path is missing or redirected')
        if path.is_file():
            if not target.is_file() or hashlib.sha256(path.read_bytes()).digest() != hashlib.sha256(target.read_bytes()).digest():
                raise ValueError('recovered install bytes differ')
        elif not path.is_dir() or not target.is_dir():
            raise ValueError('recovered install node differs')
        target.chmod(stat.S_IMODE(path.stat().st_mode))
