"""Locked input validation; never execute or extract an unverified archive."""
import hashlib
import json
import posixpath
import re
import tarfile
from pathlib import Path, PurePosixPath


class InputError(ValueError):
    pass


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load_lock(path):
    value = json.loads(Path(path).read_text())
    if value.get('schema') != 1 or value.get('architecture') != 'arm64':
        raise InputError('unexpected schema or architecture')
    if not re.fullmatch(r'debian@sha256:[0-9a-f]{64}', value['base_image']):
        raise InputError('builder must use immutable Debian manifest')
    source = value['mesa']
    if (not re.fullmatch('[0-9a-f]{40}', source['revision'])
            or source['archive'] != 'mesa-' + source['revision'] + '.tar.gz'
            or not re.fullmatch(r'\d{8}T\d{6}Z', value['snapshot'])):
        raise InputError('unsafe or mutable source/snapshot locator')
    for name in source['source_files']:
        path = PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or '\\' in name:
            raise InputError('unsafe source pin path')
    for digest in [value['mesa']['sha256'], *value['releases'].values(),
                   *value['mesa']['source_files'].values()]:
        if not re.fullmatch('[0-9a-f]{64}', digest):
            raise InputError('malformed digest')
    required = {'--wrap-mode=nofallback', '-Dgallium-drivers=virgl',
                '-Dvulkan-drivers=virtio', '-Dllvm=disabled', '-Dglx=disabled',
                '-Degl=enabled', '-Dgles2=enabled', '-Dopengl=true', '-Dgbm=enabled',
                '-Dplatforms=[]', '-Dglvnd=disabled', '-Dallow-fallback-for=[]',
                '--prefix=/opt/mesa-f02', '--libdir=lib', '--buildtype=release'}
    if not required.issubset(value['meson_args']):
        raise InputError('missing driver/build fence')
    for arg in value['meson_args']:
        if not re.fullmatch(r'(?:--[a-z-]+|-D[a-z0-9-]+)=[^\s]+', arg):
            raise InputError('Meson options must have one attached value')
    names = [arg.split('=')[0] for arg in value['meson_args']]
    if len(names) != len(set(names)):
        raise InputError('duplicate option can override build fence')
    return value


def archive_members(archive, root):
    members = archive.getmembers()
    names = {}
    for member in members:
        name = member.name.rstrip('/')
        path = PurePosixPath(name)
        if (not path.parts or '\x00' in name or '\\' in name or path.is_absolute()
                or '..' in path.parts or path.parts[0] != root or name in names):
            raise InputError('unsafe or duplicate archive path: ' + name)
        if not (member.isfile() or member.isdir() or member.issym() or member.islnk()):
            raise InputError('special archive node: ' + name)
        names[name] = member
    for name, member in names.items():
        for parent in PurePosixPath(name).parents:
            if str(parent) == '.':
                break
            if str(parent) not in names or not names[str(parent)].isdir():
                raise InputError('non-directory archive ancestor: ' + name)
        if member.issym() or member.islnk():
            target = member.linkname
            if '\\' in target or '\x00' in target or target.startswith('/'):
                raise InputError('unsafe link target: ' + name)
            target = posixpath.normpath(posixpath.join(posixpath.dirname(name), target)
                                       if member.issym() else target)
            visited = {name}
            while True:
                if not target.startswith(root + '/') or target not in names:
                    raise InputError('missing or escaping link target: ' + name)
                item = names[target]
                if not (item.issym() or item.islnk()):
                    break
                if target in visited:
                    raise InputError('cyclic archive link: ' + name)
                visited.add(target)
                target = posixpath.normpath(posixpath.join(posixpath.dirname(target), item.linkname)
                                           if item.issym() else item.linkname)
    return members


def verify_archive(path, lock):
    source = lock['mesa']
    if Path(path).stat().st_size != source['bytes'] or sha(path) != source['sha256']:
        raise InputError('Mesa archive size/hash mismatch')
    root = 'mesa-' + source['revision']
    with tarfile.open(path) as archive:
        members = archive_members(archive, root)
        for name, digest in source['source_files'].items():
            member = archive.getmember(root + '/' + name)
            if not member.isfile():
                raise InputError('source pin is not a regular file')
            if hashlib.sha256(archive.extractfile(member).read()).hexdigest() != digest:
                raise InputError('F02 source pin mismatch: ' + name)
        version = archive.extractfile(root + '/VERSION').read().decode().strip()
        if version != source['version']:
            raise InputError('Mesa version mismatch')
    return len(members)


def verify_extracted(archive_path, output, lock, source_root=None):
    output = Path(output)
    root = 'mesa-' + lock['mesa']['revision']
    directory = Path(source_root) if source_root is not None else output / root
    actual = {root, *(root + '/' + str(path.relative_to(directory)) for path in directory.rglob('*'))}
    with tarfile.open(archive_path) as archive:
        members = archive_members(archive, root)
        expected = {member.name.rstrip('/') for member in members}
        if actual != expected:
            raise InputError('source tree has added or missing paths: added=' + repr(sorted(actual - expected)[:8])
                             + ' missing=' + repr(sorted(expected - actual)[:8]))
        for member in members:
            path = directory if member.name.rstrip('/') == root else directory / member.name[len(root) + 1:]
            if member.isfile() or member.islnk():
                if path.is_symlink() or sha(path) != hashlib.sha256(archive.extractfile(member).read()).hexdigest():
                    raise InputError('source tree differs from archive: ' + member.name)
            elif member.issym():
                if not path.is_symlink() or path.readlink().as_posix() != member.linkname:
                    raise InputError('source symlink differs from archive')
            elif not path.is_dir() or path.is_symlink():
                raise InputError('source directory differs from archive')


def package_rows(directory):
    rows = []
    for path in sorted(Path(directory).glob('*.json')):
        rows.extend(json.loads(path.read_text()))
    seen = set()
    for row in rows:
        name = row['file']
        if Path(name).name != name or not name.endswith('.deb') or name in seen:
            raise InputError('unsafe or duplicate package archive')
        seen.add(name)
        if row['architecture'] not in {'arm64', 'all'}:
            raise InputError('foreign package architecture')
        if not re.fullmatch('[0-9a-f]{64}', row['sha256']):
            raise InputError('malformed package digest')
    return rows


def verify_packages(directory, rows):
    files = {path.name for path in Path(directory).glob('*.deb')}
    if files != {row['file'] for row in rows}:
        raise InputError('package set differs from lock')
    for row in rows:
        path = Path(directory) / row['file']
        if path.stat().st_size != row['bytes'] or sha(path) != row['sha256']:
            raise InputError('package size/hash mismatch: ' + row['file'])
