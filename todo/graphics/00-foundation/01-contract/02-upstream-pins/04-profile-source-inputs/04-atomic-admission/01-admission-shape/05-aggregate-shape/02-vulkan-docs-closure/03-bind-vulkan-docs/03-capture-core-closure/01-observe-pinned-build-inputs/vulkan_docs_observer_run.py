#!/usr/bin/env python3
"""Run and assemble declared observer-only Vulkan Docs replays."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from vulkan_docs_observer_model import OBSERVATION_FIELDS, RUN_FIELDS, reject
from vulkan_docs_observer_contract import observation_value, run as validate_run
from vulkan_docs_observer_normalize import normalize
from vulkan_docs_observer_parse import F02, canonical, digest_file, document, identifier, source_limit
from vulkan_docs_observer_plan import (
    DOCS_COMMIT, SOURCE_TREE, build_command, dry_run_command, observer_digest, output_identity, producer_digest, witness_digest,
)


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n", encoding="utf-8")


def run_command(command: list[str], output: Path) -> None:
    with output.open("wb") as stream:
        subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, check=True)


def source_tree(source: Path) -> tuple[int, int, str]:
    status = subprocess.run(["git", "-C", str(source), "status", "--porcelain", "--ignored"], capture_output=True, text=True, check=True)
    commit = subprocess.run(["git", "-C", str(source), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if status.stdout or commit != DOCS_COMMIT or any(source.rglob("*.pyc")):
        reject("observer source is not a clean pinned checkout without Python cache files")
    listing = subprocess.run(["git", "-C", str(source), "ls-files", "-s", "-z"], capture_output=True, check=True).stdout
    rows = []
    for row in listing.split(b"\0"):
        if not row:
            continue
        metadata, name = row.split(b"\t", 1)
        mode, _, _ = metadata.split()
        path = source / name.decode("utf-8")
        if path.is_symlink() or not path.is_file():
            reject("tracked observer source is not a regular file")
        data = path.read_bytes()
        if len(data) > F02.MAX_INPUT_BYTES:
            reject("tracked observer source exceeds the F02 member cap")
        rows.append({"selector": name.decode("utf-8"), "sha256": hashlib.sha256(data).hexdigest(),
                     "bytes": len(data), "mode": mode.decode("ascii")})
    rendered = json.dumps(rows, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")
    result = len(rows), sum(row["bytes"] for row in rows), hashlib.sha256(rendered).hexdigest()
    if result != SOURCE_TREE:
        reject("observer source tree does not retain the reviewed pinned identity")
    return result


def seal_run(value: dict[str, object]) -> dict[str, object]:
    if set(value) != RUN_FIELDS - {"run_sha256"}:
        reject("observer run cannot be sealed from an invalid schema")
    value["run_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-observation-run-v1")
    return value


def capture(identifier_value: str, source: Path, work: Path, artifact: str) -> Path:
    identifier(identifier_value, "observer run")
    source, work = source.resolve(), work.resolve()
    if work.exists() and any(work.iterdir()):
        reject("observer work directory must be new and empty")
    work.mkdir(parents=True, exist_ok=True)
    observer, generated = work / "observer", work / "generated"
    observer.mkdir()
    source_tree(source)
    run_command(dry_run_command(source, work), observer / "producer-dry-run.txt")
    run_command(build_command(source, work), observer / "build.log")
    count, total, generated_digest, html_digest = output_identity(generated)
    result = normalize(source, generated, observer / "io-events.jsonl", observer / "resolved-includes.jsonl")
    write(observer / "normalized-inputs.json", result)
    value = {
        "id": identifier_value, "artifact": artifact, "source_tree_sha256": SOURCE_TREE[2],
        "generated_tree_sha256": generated_digest, "primary_html_sha256": html_digest,
        "io_trace_sha256": digest_file(observer / "io-events.jsonl", "observer I/O trace"),
        "include_trace_sha256": digest_file(observer / "resolved-includes.jsonl", "observer include trace"),
        "input_manifest_sha256": result["input_manifest_sha256"], "include_identity_sha256": result["include_identity_sha256"],
        "producer_argv_sha256": producer_digest(),
        "raw_count": result["raw_count"], "derived_count": result["derived_count"], "include_count": len(result["includes"]),
        "phase_counts": result["phase_counts"],
    }
    if count != 2530 or total != 17019466:
        reject("observer output witness count does not match the pinned build")
    run_path = observer / "run.json"
    sealed = seal_run(value)
    validate_run(sealed)
    write(run_path, sealed)
    return run_path


def assemble(paths: list[Path], output: Path) -> None:
    rows = [document(path) for path in paths]
    value = {
        "schema": 1, "contract": "vulkan-docs-core-input-observation-v1", "status": "input-observation-only-unadmitted",
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "build_witness_sha256": witness_digest(), "observer_source_sha256": observer_digest(), "runs": rows,
    }
    if set(value) != OBSERVATION_FIELDS - {"observation_sha256"}:
        reject("observer header cannot be sealed from an invalid schema")
    value["observation_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-observation-v1")
    observation_value(value)
    write(output, value)


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    capture_parser = commands.add_parser("capture")
    capture_parser.add_argument("--id", required=True)
    capture_parser.add_argument("--source", type=Path, required=True)
    capture_parser.add_argument("--work", type=Path, required=True)
    capture_parser.add_argument("--artifact", required=True)
    assemble_parser = commands.add_parser("assemble")
    assemble_parser.add_argument("--run", type=Path, action="append", required=True)
    assemble_parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        if arguments.command == "capture":
            print(capture(arguments.id, arguments.source, arguments.work, arguments.artifact))
        else:
            assemble(arguments.run, arguments.output)
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"FAIL: {error}")
        raise SystemExit(2)


if __name__ == "__main__":
    main()
