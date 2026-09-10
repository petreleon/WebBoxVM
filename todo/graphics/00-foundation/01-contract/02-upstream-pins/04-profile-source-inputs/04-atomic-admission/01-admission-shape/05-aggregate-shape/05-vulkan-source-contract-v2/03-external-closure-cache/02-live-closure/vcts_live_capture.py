#!/usr/bin/env python3
"""Capture one immutable VCTS closure, replay it locally, and prove it offline."""

from __future__ import annotations

import argparse
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
for directory in (HERE, CACHE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))
import vcts_cache_contract as cache
import vcts_capture_json as output
import vcts_capture_receipt as receipt
import vcts_github_tree as github
import vcts_live_publish as publish
import vcts_live_stage as stage
import vcts_tree_plan as plan

class LiveCaptureError(ValueError):
    """A live closure cannot be safely captured, replayed, or recorded."""

def reject(message: str) -> None:
    raise LiveCaptureError(message)

@dataclass(frozen=True)
class CaptureResult:
    plan_sha256: str
    ledger_sha256: str
    member_count: int
    member_total_bytes: int
    required_free_bytes: int

def _ancestor(path: Path) -> Path:
    candidate = path
    while not candidate.exists() and candidate != candidate.parent:
        candidate = candidate.parent
    if not candidate.is_dir():
        reject("external cache has no usable filesystem ancestor")
    return candidate

def required_space(tree: dict[str, object]) -> int:
    members = tree.get("members")
    total = tree.get("member_total_bytes")
    if not isinstance(members, list) or type(total) is not int or not members:
        reject("tree plan cannot establish capture storage requirements")
    sizes = [row.get("bytes") for row in members if isinstance(row, dict)]
    if len(sizes) != len(members) or any(type(item) is not int or item < 1 for item in sizes):
        reject("tree plan has invalid member storage requirements")
    root = plan.identity.ROOT["bytes"]
    return 2 * (root + total) + max(root, *sizes) + 8 * 1024 * 1024

def _ledger(root, members) -> dict[str, object]:
    rows = [member.ledger_row() for member in members]
    value: dict[str, object] = {
        "schema": 1, "kind": "canonical-upstream-suite", "root_identity_sha256": root.digest,
        "limits": dict(cache.ledger.LIMITS), "member_count": len(rows),
        "member_total_bytes": sum(row["bytes"] for row in rows), "ledger_sha256": "0" * 64,
        "members": rows,
    }
    value["ledger_sha256"] = cache.ledger.digest(value)
    return value

def _marker(root: Path, identity_digest: str, ledger: dict[str, object]) -> Path:
    return (root / "webboxvm-graphics" / "v2" / "vulkan-cts-mustpass" / identity_digest /
            str(ledger["ledger_sha256"]) / cache.receipt.name(ledger))

def _write(path: Path, value: dict[str, object]) -> None:
    try:
        output.write_json(path, value)
    except OSError as error:
        reject(f"temporary capture metadata write failed (errno {error.errno}, path {error.filename or path}): {error.strerror}")

def _discard(captured, failure: BaseException) -> None:
    try:
        captured.discard()
    except stage.StageError as cleanup:
        reject(f"live capture failed ({failure}); live stage cleanup failed: {cleanup}")

def capture(identity_path: Path, external_root: Path, plan_output: Path, ledger_output: Path,
            receipt_output: Path, timeout: float = 60.0, metadata_opener=None, raw_opener=None,
            tree_capture=None) -> CaptureResult:
    """Fetch metadata and raw content once, then publish only verified metadata."""
    outputs = (plan_output, ledger_output, receipt_output)
    transaction = None
    try:
        transaction = publish.preflight(outputs)
        root = cache.identity.validate(identity_path)
        observed = tree_capture or github.capture(identity_path, timeout, metadata_opener)
        tree = observed.tree_plan
        need = required_space(tree)
        try:
            free = shutil.disk_usage(_ancestor(external_root)).free
        except OSError as error:
            reject(f"external cache free space cannot be checked: {error}")
        if free < need:
            reject(f"external cache lacks required free space ({need} bytes)")
        repository = cache.repository_root(HERE)
        with tempfile.TemporaryDirectory(dir="/private/tmp") as temporary:
            temporary = Path(temporary)
            plan_path, ledger_path, receipt_path = (temporary / "tree-plan.json", temporary / "ledger.json",
                                                    temporary / "live-receipt.json")
            _write(plan_path, tree)
            plan.validate(plan_path, identity_path)
            captured = None
            try:
                captured = stage.capture(identity_path, plan_path, external_root, repository, timeout, raw_opener)
                ledger = _ledger(root, captured.members)
                _write(ledger_path, ledger)
                closure = cache.ledger.validate(ledger_path, identity_path)
                with captured.replay() as local:
                    populated = cache.populate(identity_path, ledger_path, external_root, repository, timeout, local)
                    local.assert_complete()
                if populated.reused:
                    reject("fresh capture unexpectedly reused an existing cache closure")
                checked = cache.verify(identity_path, ledger_path, external_root, repository)
                if checked.digest != closure.digest:
                    reject("offline cache verification changed the closure digest")
                marker = _marker(external_root, root.digest, ledger)
                streams = {"root_streams": 1, "member_streams": closure.member_count,
                           "total_streams": closure.member_count + 1, "raw_https_requests": closure.member_count + 1,
                           "sha256_verified_streams": closure.member_count + 1,
                           "git_blob_sha1_verified_streams": closure.member_count + 1,
                           "total_bytes": captured.total_bytes}
                offline = {"status": "passed", "verify_calls": 1, "raw_https_requests": 0,
                           "member_count": closure.member_count, "member_total_bytes": closure.total_bytes,
                           "cache_receipt_sha256": cache.receipt.parse(marker.read_bytes())["receipt_sha256"]}
                value = receipt.build(identity_path, plan_path, ledger_path, marker,
                                      {"sha256": captured.root.sha256, "git_blob_sha1": captured.root.blob_sha1,
                                       "bytes": captured.root.bytes}, streams, offline)
                _write(receipt_path, value)
                receipt.validate(receipt_path, identity_path, plan_path, ledger_path, marker)
                try:
                    captured.discard()
                finally:
                    captured = None
                publish.publish(transaction, tuple(output.render(item).encode("utf-8") for item in (tree, ledger, value)))
                return CaptureResult(tree["plan_sha256"], closure.digest, closure.member_count,
                                     closure.total_bytes, need)
            except BaseException as failure:
                if captured is not None:
                    _discard(captured, failure)
                raise
    except (cache.identity.IdentityError, cache.ledger.LedgerError, cache.io.CacheError,
            github.GitHubTreeError, plan.TreePlanError, receipt.CaptureReceiptError, publish.PublishError) as error:
        reject(str(error))
    except OSError as error:
        reject(f"live capture filesystem operation failed (errno {error.errno}, path {error.filename or '<unknown>'}): {error.strerror}")
    finally:
        if transaction is not None:
            transaction.close()

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True,
                        help="fresh current-user 0700 directory for this immutable output bundle")
    parser.add_argument("--identity", type=Path, default=SCHEMA / "vcts_root_identity.json")
    parser.add_argument("--timeout", type=float, default=60.0)
    args = parser.parse_args()
    try:
        result = capture(args.identity, args.cache_root, *(args.output_dir / name for name in
                         ("vcts_tree_plan.json", "vcts_closure_ledger.json", "vcts_live_capture_receipt.json")), args.timeout)
    except LiveCaptureError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    except OSError as error:
        print(f"FAIL: filesystem operation failed (errno {error.errno}, path {error.filename or '<unknown>'}): {error.strerror}", file=sys.stderr)
        raise SystemExit(2)
    print(f"PASS: {result.member_count} members {result.member_total_bytes} bytes; unadmitted VCTS input")
if __name__ == "__main__":
    main()
