"""Atomic graphics-chunk bundle output and stale-output checks."""

from __future__ import annotations

import os
import shutil
import tempfile
from collections.abc import Callable
from pathlib import Path


Failure = Callable[[str], None]


def write_bundle(output: Path, bundle: dict[str, bytes], fail: Failure) -> None:
    """Atomically replace an output directory with the exact expected bundle."""
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{output.name}.stage-", dir=output.parent))
    backup = output.with_name(f".{output.name}.backup-{os.getpid()}")
    try:
        for name, data in bundle.items():
            (stage / name).write_bytes(data)
        if output.exists():
            if not output.is_dir() or backup.exists():
                fail(f"cannot atomically replace {output}")
            output.replace(backup)
        stage.replace(output)
        if backup.exists():
            shutil.rmtree(backup)
    except BaseException:
        if not output.exists() and backup.exists():
            backup.replace(output)
        raise
    finally:
        if stage.exists():
            shutil.rmtree(stage)


def check_bundle(output: Path, bundle: dict[str, bytes], fail: Failure) -> None:
    """Reject a missing, extra, or byte-stale generated output bundle."""
    if not output.is_dir():
        fail(f"generated output is missing: {output}")
    actual = {path.name for path in output.iterdir() if path.is_file()}
    expected = set(bundle)
    if actual != expected:
        fail(f"generated output file set is stale: expected {sorted(expected)}, got {sorted(actual)}")
    for name, data in bundle.items():
        if (output / name).read_bytes() != data:
            fail(f"generated output is stale: {name}")
