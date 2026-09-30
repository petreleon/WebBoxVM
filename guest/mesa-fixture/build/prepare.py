"""Capture signed snapshot packages, freeze hashes, and build an offline image."""
import hashlib
import json
import pathlib
import os
import sys
import shutil
import subprocess
import tarfile
import urllib.request

from inputs import load_lock, package_rows, verify_archive, verify_extracted, verify_packages

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
INPUT = ROOT / '.artifacts/graphics/i01-mesa-image'
OUT = INPUT / 'build'
ATTEMPT = os.environ.get('I01_CAPTURE_ATTEMPT', '01')
NAME = 'webboxvm-i01-mesa-package-capture-' + ATTEMPT
TAG = 'webboxvm/i01-mesa-builder:06f9e283-snapshot20260901'


def run(args, stage):
    path = OUT / 'logs' / (stage + '.log')
    if path.exists():
        raise SystemExit('preserve prior log; choose a fresh I01_CAPTURE_ATTEMPT')
    with path.open('wb') as log:
        result = subprocess.run(args, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit(f'{stage}: first failure exit {result.returncode}; inspect log')


def releases(lock):
    rows = []
    for origin, digest in lock['releases'].items():
        suite = 'trixie-security' if origin.endswith('security') else 'trixie'
        url = f'https://snapshot.debian.org/archive/{origin}/{lock["snapshot"]}/dists/{suite}/Release'
        with urllib.request.urlopen(url, timeout=120) as response:
            raw = response.read()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise SystemExit('snapshot Release differs from lock: ' + origin)
        (OUT / 'manifests' / (origin + '-Release')).write_bytes(raw)
        rows.append({'url': url, 'bytes': len(raw), 'sha256': digest})
    (OUT / 'manifests/releases.json').write_text(json.dumps(rows, indent=2) + '\n')


def freeze_packages():
    rows = []
    for line in (OUT / 'manifests/downloaded-packages.tsv').read_text().splitlines():
        name, size, package, version, architecture, digest = line.split('\t')
        rows.append({'file': name, 'package': package, 'version': version,
                     'architecture': architecture, 'bytes': int(size), 'sha256': digest})
    directory = HERE / 'packages'
    if directory.exists():
        locked = package_rows(directory)
        if locked != rows:
            raise SystemExit('resolved packages differ from maintained package lock')
    else:
        directory.mkdir()
        for offset in range(0, len(rows), 60):
            path = directory / f'{offset // 60 + 1:02}.json'
            path.write_text('[\n' + ',\n'.join('  ' + json.dumps(row) for row in rows[offset:offset + 60]) + '\n]\n')
    verify_packages(OUT / 'packages', rows)
    (OUT / 'manifests/packages.json').write_text(json.dumps(rows, indent=2) + '\n')
    return rows


def main():
    lock = load_lock(HERE / 'lock.json')
    for name in ['logs', 'manifests']:
        (OUT / name).mkdir(parents=True, exist_ok=True)
    verify_archive(INPUT / lock['mesa']['archive'], lock)
    releases(lock)
    source = OUT / ('mesa-' + lock['mesa']['revision'])
    with tarfile.open(INPUT / lock['mesa']['archive']) as archive:
        if source.exists():
            verify_extracted(INPUT / lock['mesa']['archive'], OUT, lock)
        else:
            archive.extractall(OUT, filter='data')
    if '--resume-captured-packages' not in sys.argv:
        run(['docker', 'run', '--platform', 'linux/arm64', '--name', NAME,
             '-v', f'{OUT}:/output', '-v', f'{HERE}:/recipe:ro', lock['base_image'],
             'sh', '/recipe/capture.sh', *lock['packages']], '01-capture-' + ATTEMPT)
    rows = freeze_packages()
    context = OUT / ('builder-context-' + ATTEMPT)
    context.mkdir()
    shutil.copy2(HERE / 'Dockerfile', context / 'Dockerfile')
    shutil.copytree(OUT / 'packages', context / 'packages', ignore=shutil.ignore_patterns('partial', 'lock'))
    (context / 'packages.sha256').write_text(''.join(row['sha256'] + '  ' + row['file'] + '\n' for row in rows))
    run(['docker', 'build', '--platform', 'linux/arm64', '--network=none',
         '-t', TAG, str(context)], '02-builder-' + ATTEMPT)
    identity = subprocess.check_output(['docker', 'image', 'inspect', TAG, '--format', '{{.Id}}']).decode().strip()
    (OUT / 'manifests/builder.json').write_text(json.dumps({'image_tag': TAG, 'image_id': identity,
         'base_image': lock['base_image'], 'snapshot': lock['snapshot'], 'native_architecture': 'arm64'}, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'packages': len(rows), 'builder': TAG, 'source': str(source)}))


if __name__ == '__main__':
    main()
