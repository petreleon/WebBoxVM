"""Copy a finite ARM64 ELF dependency closure without software driver fallback."""
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess

from paths import validate_links
from layout import driver_layout

OUT = pathlib.Path(os.environ.get('I01_WORK_ROOT', '/work'))
DEST = OUT / 'runtime-rootfs'
PREFIX = pathlib.Path('/opt/mesa-f02')
RUNTIME_LIBRARIES = ('libvulkan.so.1',)


def output(args):
    return subprocess.check_output(args, text=True)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def elf(path):
    try:
        return path.is_file() and path.open('rb').read(4) == b'\x7fELF'
    except OSError:
        return False


def dynamic(path):
    header = output(['readelf', '-h', str(path)])
    if 'AArch64' not in header:
        raise ValueError('non-ARM64 ELF: ' + str(path))
    text = output(['readelf', '-d', str(path)])
    needed = re.findall(r'\(NEEDED\).*\[(.*?)\]', text)
    soname = re.findall(r'\(SONAME\).*\[(.*?)\]', text)
    program = output(['readelf', '-l', str(path)])
    interpreter = re.findall(r'Requesting program interpreter: (.*?)\]', program)
    return needed, soname, interpreter


def copy(path):
    path = pathlib.Path(os.path.normpath(str(path)))
    if not path.is_absolute() or '..' in path.parts:
        raise ValueError('unsafe runtime input path')
    target = DEST / str(path).lstrip('/')
    target.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink():
        link = os.readlink(path)
        if target.is_symlink():
            if os.readlink(target) != link:
                raise ValueError('runtime symlink collision')
        elif not target.exists():
            target.symlink_to(link)
        # Preserve the logical target too: /lib may alias /usr/lib in the builder.
        return copy(pathlib.Path(link) if link.startswith('/') else path.parent / link)
    if not target.exists():
        shutil.copy2(path, target)
    return path


def seed_libraries(providers, queue):
    # vulkaninfo uses dlopen; DT_NEEDED alone omits the loader required by clients.
    for name in RUNTIME_LIBRARIES:
        if name not in providers:
            raise ValueError('missing dynamic runtime library: ' + name)
        queue.append(copy(providers[name]).resolve())


def main():
    if DEST.exists():
        raise SystemExit('runtime-rootfs already exists; preserve prior build')
    shutil.copytree(OUT / 'mesa-destdir', DEST, symlinks=True)
    providers = {}
    queue = []
    for path in PREFIX.rglob('*'):
        if elf(path):
            real = path.resolve()
            queue.append(real)
            providers[path.name] = real
            for soname in dynamic(real)[1]:
                providers[soname] = real
    for line in output(['ldconfig', '-p']).splitlines():
        match = re.match(r'\s*(\S+) .*=> (\S+)', line)
        if match:
            providers.setdefault(match[1], pathlib.Path(match[2]))
    tool_names = ['eglinfo', 'glxinfo', 'vulkaninfo', 'busybox', 'modprobe', 'depmod', 'xz', 'sh']
    tools = {}
    seed_libraries(providers, queue)
    for name in tool_names:
        path = shutil.which(name)
        if path is None:
            # Debian mesa-utils-bin installs architecture-suffixed executables.
            candidates = sorted(pathlib.Path('/usr/bin').glob(name + '.*-linux-gnu'))
            if len(candidates) != 1:
                raise ValueError('missing or ambiguous standard tool: ' + name)
            path = str(candidates[0])
        real = copy(path)
        queue.append(real)
        tools[name] = str(path)
        if pathlib.Path(path).name != name:
            alias = DEST / 'usr/bin' / name
            alias.symlink_to(pathlib.Path(path).name)
    edges, seen = [], set()
    while queue:
        path = queue.pop()
        if path in seen or not elf(path):
            continue
        seen.add(path)
        needed, _, interpreters = dynamic(path)
        for name in needed:
            if name not in providers:
                raise ValueError('unresolved ELF dependency: ' + name)
            provider = providers[name]
            copy(provider)
            queue.append(provider.resolve())
            edges.append({'elf': str(path), 'needed': name, 'provider': str(provider)})
        for interpreter in interpreters:
            queue.append(copy(interpreter))
    # Busybox provides a compact init shell and standard early-userspace applets.
    applets = ['mount', 'mkdir', 'cat', 'echo', 'sleep', 'uname', 'ls', 'dmesg', 'poweroff', 'stty', 'base64']
    available = output(['busybox', '--list']).splitlines()
    if not set(applets).issubset(available):
        raise ValueError('required early-userspace busybox applet missing')
    for name in applets:
        target = DEST / 'bin' / name
        target.parent.mkdir(exist_ok=True)
        if not target.exists():
            target.symlink_to('/usr/bin/busybox')
    shell = DEST / 'bin/sh'
    if not shell.exists():
        shell.symlink_to(tools['sh'])
    validate_links(DEST)
    lock = json.loads(pathlib.Path('/recipe/lock.json').read_text())
    layout = driver_layout(DEST / 'opt/mesa-f02', lock['mesa']['version'])
    rows = []
    size = 0
    for path in sorted(DEST.rglob('*')):
        rel = '/' + str(path.relative_to(DEST))
        if path.is_symlink():
            rows.append({'path': rel, 'symlink': os.readlink(path)})
        elif path.is_file():
            size += path.stat().st_size
            rows.append({'path': rel, 'bytes': path.stat().st_size, 'sha256': digest(path)})
    if size >= lock['runtime_limit_bytes']:
        raise ValueError('runtime closure exceeds size budget')
    manifest = {'schema': 1, 'bytes': size, 'tools': tools, 'dynamic_libraries': list(RUNTIME_LIBRARIES), 'drivers': layout['drivers'],
                'gallium_library': '/opt/mesa-f02/lib/libgallium-' + lock['mesa']['version'] + '.so',
                'legacy_dri_aliases': [], 'icds': [str(pathlib.Path(path).relative_to(DEST)) for path in layout['icds']], 'elf_dependencies': edges,
                'files': rows, 'runtime_success_claimed': False}
    (OUT / 'manifests/runtime.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'bytes': size, 'files': len(rows), 'ELFs': len(seen)}))


if __name__ == '__main__':
    main()
