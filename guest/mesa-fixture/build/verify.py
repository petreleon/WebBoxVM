"""Verify pinned inputs and the complete unmodified extracted source tree."""
import json
import pathlib
import sys

from inputs import load_lock, package_rows, sha, verify_archive, verify_extracted, verify_packages

HERE = pathlib.Path(__file__).resolve().parent
INPUT = HERE.parents[2] / '.artifacts/graphics/i01-mesa-image'
OUT = INPUT / 'build'


def main():
    lock = load_lock(HERE / 'lock.json')
    archive = INPUT / lock['mesa']['archive']
    count = verify_archive(archive, lock)
    tree_checked = '--input-only' not in sys.argv
    if tree_checked:
        verify_extracted(archive, OUT, lock)
    rows = package_rows(HERE / 'packages')
    verify_packages(OUT / 'packages', rows)
    print(json.dumps({'result': 'PASS', 'archive_members': count, 'packages': len(rows),
                      'archive_sha256': sha(archive), 'source_tree_checked': tree_checked,
                      'source_modified': False if tree_checked else None}))


if __name__ == '__main__':
    main()
