"""Resolve runtime links in a guest root, never through the host root."""
from pathlib import Path, PurePosixPath


def rooted_path(root, name):
    root = Path(root)
    pending = list(PurePosixPath('/' + str(name).lstrip('/')).parts[1:])
    resolved, links = [], 0
    while pending:
        part = pending.pop(0)
        if part == '.':
            continue
        if part == '..':
            if not resolved:
                raise ValueError('runtime link escapes guest root')
            resolved.pop()
            continue
        path = root.joinpath(*resolved, part)
        if path.is_symlink():
            links += 1
            if links > 40:
                raise ValueError('cyclic runtime link')
            target = path.readlink().as_posix()
            pieces = list(PurePosixPath(target).parts)
            if target.startswith('/'):
                resolved = []
                pieces = pieces[1:]
            pending = pieces + pending
        else:
            resolved.append(part)
    return root.joinpath(*resolved)


def validate_links(root):
    root = Path(root)
    for path in root.rglob('*'):
        if path.is_symlink() and not rooted_path(root, path.relative_to(root)).exists():
            raise ValueError('missing guest runtime link target: ' + str(path.relative_to(root)))
