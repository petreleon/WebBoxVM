"""Runtime-only recovery from a sealed successful compile/install receipt."""
import hashlib
import json
import os
import pathlib
import shutil

import compile as compiler
from export import export_tree, preserve_install_modes
from inputs import load_lock, verify_extracted


def main():
    local, host = compiler.LOCAL, compiler.HOST
    sealed = pathlib.Path(os.environ.get('I01_SEALED_BUILD', str(host)))
    for name in ['logs', 'manifests']:
        (local / name).mkdir(exist_ok=True)
    lock = load_lock('/recipe/lock.json')
    receipt_path = sealed / 'manifests/execution-04.json'
    receipt_raw = receipt_path.read_bytes()
    receipt = json.loads(receipt_raw)
    if receipt.get('source_before_compile') != 'PASS' or receipt.get('source_after_compile') != 'PASS':
        raise ValueError('missing successful unmodified-source proof')
    stages = {row['stage']: row for row in receipt['stages']}
    expected = ['meson', 'setup', compiler.BUILD, '/source', *lock['meson_args']]
    if stages['10-configure']['command'] != expected:
        raise ValueError('original configured flags differ from current source lock')
    for stage in ['10-configure', '11-compile', '12-install']:
        if stages[stage]['exit'] != 0:
            raise ValueError('original build/install did not complete')
    compiler.RECEIPT['compiled_execution_receipt_sha256'] = hashlib.sha256(receipt_raw).hexdigest()
    compiler.RECEIPT['recovery_scope'] = 'setup-introspection-and-runtime-only-no-ninja'
    env = os.environ.copy()
    env.update(SOURCE_DATE_EPOCH='1767225600', PYTHONHASHSEED='0',
               PYTHONDONTWRITEBYTECODE='1', I01_WORK_ROOT='/work')
    try:
        archive = compiler.source(lock)
        compiler.run(expected, '14-reconfigure-for-introspection', env)
        verify_extracted(archive, local, lock, source_root='/source')
        (local / 'mesa-destdir/opt').mkdir(parents=True)
        shutil.copytree('/opt/mesa-f02', local / 'mesa-destdir/opt/mesa-f02', symlinks=True)
        compiler.run(['python3', '/recipe/runtime.py'], '15-runtime-recovery', env)
        compiler.manifest(lock, env)
        path = local / 'manifests/build.json'
        manifest = json.loads(path.read_text())
        manifest['compiled_execution_receipt_sha256'] = hashlib.sha256(receipt_raw).hexdigest()
        manifest['compiled_execution_receipt'] = 'execution-04.json'
        manifest['runtime_recovery_scope'] = 'same-compiled-install-no-ninja'
        path.write_text(json.dumps(manifest, indent=2) + '\n')
        compiler.RECEIPT['complete'] = True
    except BaseException as error:
        compiler.RECEIPT['error'] = str(error)
        raise
    finally:
        path = local / 'manifests' / ('runtime-recovery-' + compiler.ATTEMPT + '.json')
        path.write_text(json.dumps(compiler.RECEIPT, indent=2) + '\n')
        for name in ['logs', 'manifests']:
            for path in (local / name).iterdir():
                target = host / name / path.name
                if target.exists():
                    raise ValueError('preserve prior result: ' + str(target))
                shutil.copy2(path, target)
        if compiler.RECEIPT.get('complete'):
            export_tree(local / 'runtime-rootfs', host / 'runtime-rootfs')
            preserve_install_modes(sealed / 'mesa-destdir', host / 'runtime-rootfs')


if __name__ == '__main__':
    main()
