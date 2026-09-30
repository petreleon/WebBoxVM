"""Compile sealed sources in bounded container-local storage; export only results."""
import copy
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import tarfile
import time

from inputs import load_lock, verify_archive, verify_extracted
from export import export_tree

LOCAL = pathlib.Path('/work')
HOST = pathlib.Path('/output')
BUILD = '/work/mesa-build'
ATTEMPT = os.environ.get('I01_BUILD_ATTEMPT', '01')
RECEIPT = {'schema': 1, 'attempt': ATTEMPT, 'build_directory': BUILD, 'stages': []}


def run(args, name, env):
    start = time.monotonic()
    path = LOCAL / 'logs' / (name + '-' + ATTEMPT + '.log')
    with path.open('wb') as log:
        result = subprocess.run(args, stdout=log, stderr=subprocess.STDOUT, env=env)
    row = {'stage': name, 'command': args, 'exit': result.returncode,
           'seconds': round(time.monotonic() - start, 3)}
    RECEIPT['stages'].append(row)
    print(json.dumps(row), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)


def source(lock):
    archive_path = LOCAL / lock['mesa']['archive']
    shutil.copyfile('/input/archive.tar.gz', archive_path)
    verify_archive(archive_path, lock)
    prefix = 'mesa-' + lock['mesa']['revision']
    with tarfile.open(archive_path) as archive:
        members = []
        for member in archive.getmembers():
            if member.name.rstrip('/') == prefix:
                continue
            item = copy.copy(member)
            item.name = member.name[len(prefix) + 1:]
            if item.islnk():
                item.linkname = item.linkname[len(prefix) + 1:]
            members.append(item)
        archive.extractall('/source', members=members, filter='data')
    verify_extracted(archive_path, LOCAL, lock, source_root='/source')
    RECEIPT['source_before_compile'] = 'PASS'
    return archive_path


def manifest(lock, env):
    options = subprocess.check_output(['meson', 'introspect', BUILD, '--buildoptions'])
    (LOCAL / 'manifests/meson-buildoptions.json').write_bytes(options)
    versions = {name: subprocess.check_output(command, text=True).splitlines()[0]
                for name, command in {'gcc': ['gcc', '--version'], 'meson': ['meson', '--version'],
                                      'ninja': ['ninja', '--version'], 'python': ['python3', '--version']}.items()}
    recipes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
               for path in pathlib.Path('/recipe').glob('*') if path.is_file()}
    value = {'schema': 1, 'source': lock['mesa'], 'base_image': lock['base_image'],
             'snapshot': lock['snapshot'], 'meson_args': lock['meson_args'], 'tools': versions,
             'recipe_sha256': recipes, 'source_date_epoch': env['SOURCE_DATE_EPOCH'],
             'source_before_compile': 'PASS', 'source_after_compile': 'PASS',
             'network': 'none', 'source_patched': False, 'guest_runtime_validated': False,
             'clean_build_reproduction_validated': False}
    (LOCAL / 'manifests/build.json').write_text(json.dumps(value, indent=2) + '\n')


def export():
    (LOCAL / 'manifests' / ('execution-' + ATTEMPT + '.json')).write_text(json.dumps(RECEIPT, indent=2) + '\n')
    for name in ['logs', 'manifests']:
        for path in (LOCAL / name).iterdir():
            target = HOST / name / path.name
            if target.exists():
                raise ValueError('preserve existing output: ' + str(target))
            shutil.copy2(path, target)
    if RECEIPT.get('compiled_install_complete'):
        export_tree(LOCAL / 'mesa-destdir', HOST / 'mesa-destdir')
    if RECEIPT.get('complete'):
        export_tree(LOCAL / 'runtime-rootfs', HOST / 'runtime-rootfs')


def main():
    lock = load_lock('/recipe/lock.json')
    for name in ['logs', 'manifests']:
        (LOCAL / name).mkdir(exist_ok=True)
    if subprocess.check_output(['dpkg', '--print-architecture']).strip() != b'arm64':
        raise SystemExit('builder architecture mismatch')
    env = os.environ.copy()
    env.update(SOURCE_DATE_EPOCH='1767225600', PYTHONHASHSEED='0',
               PYTHONDONTWRITEBYTECODE='1', I01_WORK_ROOT='/work')
    try:
        archive_path = source(lock)
        run(['meson', 'setup', BUILD, '/source', *lock['meson_args']], '10-configure', env)
        run(['ninja', '-C', BUILD, '-j', '4'], '11-compile', env)
        verify_extracted(archive_path, LOCAL, lock, source_root='/source')
        RECEIPT['source_after_compile'] = 'PASS'
        env['DESTDIR'] = '/work/mesa-destdir'
        run(['meson', 'install', '-C', BUILD, '--no-rebuild'], '12-install', env)
        shutil.copy2('/builder-packages.tsv', LOCAL / 'manifests/builder-packages.tsv')
        shutil.copytree(LOCAL / 'mesa-destdir/opt/mesa-f02', '/opt/mesa-f02')
        RECEIPT['compiled_install_complete'] = True
        manifest(lock, env)
        run(['python3', '/recipe/runtime.py'], '13-runtime', env)
        RECEIPT['complete'] = True
    except BaseException as error:
        RECEIPT['error'] = str(error)
        raise
    finally:
        export()


if __name__ == '__main__':
    main()
