"""Load the role-aware source contract only through its raw-byte lock."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

import role_aware_source_contract as contract
import role_aware_source_evidence as evidence

HERE = Path(__file__).resolve().parent
CONTRACT = HERE / "source_contract.json"
LOCK = HERE / "source_contract.lock"
HEADER = "webboxvm-f0254-source-contract-lock-v1"
SHA256 = re.compile(r"^[0-9a-f]{64}$")


class SourceLockError(ValueError):
    """The sealed source contract cannot be read with its exact raw identity."""


def reject(message: str) -> None:
    raise SourceLockError(message)


def regular(path: Path, label: str) -> bytes:
    if not isinstance(path, Path) or path.is_symlink() or not path.is_file():
        reject(f"{label} must be a regular nonsymlink file")
    try:
        return path.read_bytes()
    except OSError as error:
        reject(f"{label} cannot be read: {error}")


def lock_text(contract_path: Path, payload: bytes) -> bytes:
    return f"{HEADER}\n{contract_path.name} sha256={hashlib.sha256(payload).hexdigest()}\n".encode("ascii")


def expected_digest(contract_path: Path, lock: bytes) -> str:
    try:
        lines = lock.decode("ascii").splitlines()
    except UnicodeDecodeError as error:
        reject(f"source lock is not ASCII: {error}")
    prefix = f"{contract_path.name} sha256="
    if len(lines) != 2 or lines[0] != HEADER or not lines[1].startswith(prefix):
        reject("source lock has an invalid schema")
    digest = lines[1].removeprefix(prefix)
    if not SHA256.fullmatch(digest):
        reject("source lock has an invalid SHA-256")
    return digest


def load_locked_contract(contract_path: Path = CONTRACT, lock_path: Path = LOCK) -> dict[str, object]:
    payload = regular(contract_path, "source contract")
    if hashlib.sha256(payload).hexdigest() != expected_digest(contract_path, regular(lock_path, "source lock")):
        reject("source contract raw bytes differ from the sealed lock")
    try:
        value = json.loads(payload)
    except json.JSONDecodeError as error:
        reject(f"source contract is not JSON: {error}")
    try:
        contract.validate_seal(value)
    except (contract.SourceContractError, evidence.EvidenceError) as error:
        reject(str(error))
    return contract.contract()
