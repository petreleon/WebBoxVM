"""Compile startup probes only inside the pinned offline ARM64 Mesa builder."""
import argparse
import hashlib
import json
import os
import pathlib
import shlex
import subprocess

PREFIX = pathlib.Path('/opt/mesa-f02')
REVISION = '06f9e28304d5d3f109c33535c1c25b9df5769af2'
FLAGS = ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-Wformat=2',
         '-Wconversion', '-Wshadow', '-ffile-prefix-map=/probes=guest/mesa-fixture/probes']


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(args, path, environment):
    with path.open('xb') as stream:
        result = subprocess.run(args, env=environment, stdout=stream, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit('command failed exit=' + str(result.returncode) + ' log=' + str(path))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--mesa-manifest', required=True, type=pathlib.Path)
    parser.add_argument('--output', required=True, type=pathlib.Path)
    args = parser.parse_args()
    if subprocess.check_output(['dpkg', '--print-architecture']).strip() != b'arm64':
        raise SystemExit('probe builder is not native ARM64')
    mesa = json.loads(args.mesa_manifest.read_text())
    if (mesa['source']['revision'] != REVISION or mesa['source']['version'] != '25.3.6' or
            mesa['source_patched'] or mesa['network'] != 'none'):
        raise SystemExit('probe Mesa provenance mismatch')
    output = args.output
    if output.is_symlink() or (output.exists() and any(output.iterdir())):
        raise SystemExit('probe output must be an empty real directory')
    output.mkdir(parents=True, exist_ok=True)
    binaries = output / 'rootfs/usr/bin'
    binaries.mkdir(parents=True)
    logs = output / 'logs'
    logs.mkdir()
    source = pathlib.Path(__file__).resolve().parent
    environment = os.environ.copy()
    environment.update(PKG_CONFIG_PATH=str(PREFIX / 'lib/pkgconfig'),
                       LD_LIBRARY_PATH=str(PREFIX / 'lib'), SOURCE_DATE_EPOCH='1767225600')
    for name in ['egl', 'glesv2', 'gbm']:
        prefix = subprocess.check_output(['pkg-config', '--variable=prefix', name],
                                         env=environment, text=True).strip()
        if prefix != str(PREFIX):
            raise SystemExit('probe would link against noncanonical ' + name)
    definitions = [('webboxvm-mesa-gles', 'gles-startup.c', ['egl', 'glesv2', 'gbm', 'libdrm']),
                   ('webboxvm-mesa-vulkan', 'vulkan-startup.c', ['vulkan', 'libdrm'])]
    builds = []
    for name, filename, packages in definitions:
        options = shlex.split(subprocess.check_output(
            ['pkg-config', '--cflags', '--libs', *packages], env=environment, text=True))
        args_compile = ['gcc', *FLAGS, str(source / filename), '-o', str(binaries / name),
                        *options, '-Wl,-rpath,/opt/mesa-f02/lib']
        command(args_compile, logs / (name + '-compile.log'), environment)
        command(['readelf', '-h', '-d', str(binaries / name)], logs / (name + '-elf.log'), environment)
        builds.append({'binary': '/usr/bin/' + name, 'sha256': digest(binaries / name),
                       'bytes': (binaries / name).stat().st_size, 'command': args_compile})
    manifest = {'schema': 1, 'mesa_build_sha256': digest(args.mesa_manifest),
                'sources': {path.name: digest(path) for path in sorted(source.iterdir())
                            if path.is_file()}, 'flags': FLAGS, 'binaries': builds,
                'gcc': subprocess.check_output(['gcc', '--version'], text=True).splitlines()[0],
                'pkg_config': subprocess.check_output(['pkg-config', '--version'], text=True).strip(),
                'guest_runtime_validated': False}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'scope': 'native ARM64 probe compilation', 'binaries': builds}))


if __name__ == '__main__':
    main()
