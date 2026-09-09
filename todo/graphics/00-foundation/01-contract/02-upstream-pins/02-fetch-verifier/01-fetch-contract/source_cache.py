"""Network and atomic-cache operations for declared F02 source inputs."""

from __future__ import annotations

import hashlib
import os
import tempfile
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import HTTPRedirectHandler, Request, build_opener

from source_model import ContractError, ExternalCache, SourceInput, reject

CHUNK_SIZE = 64 * 1024


class DenyRedirect(HTTPRedirectHandler):
    """Do not let urllib silently substitute a different immutable URL."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        reject(f"redirect denied ({code}) to {newurl}")


def verify_payload(source: SourceInput, payload: bytes) -> None:
    if not isinstance(payload, bytes):
        reject(f"input {source.identifier} response is not bytes")
    if len(payload) != source.byte_count:
        reject(f"input {source.identifier} byte count mismatch")
    if hashlib.sha256(payload).hexdigest() != source.sha256:
        reject(f"input {source.identifier} sha256 mismatch")


def response_bytes(source: SourceInput, response) -> bytes:
    try:
        final_url, status = response.geturl(), response.getcode()
    except AttributeError:
        reject(f"input {source.identifier} response metadata is invalid")
    if final_url != source.url:
        reject(f"input {source.identifier} response URL changed")
    if status not in (None, 200):
        reject(f"input {source.identifier} returned HTTP {status}")
    chunks, total = [], 0
    while total <= source.byte_count:
        chunk = response.read(min(CHUNK_SIZE, source.byte_count + 1 - total))
        if not chunk:
            break
        if not isinstance(chunk, bytes):
            reject(f"input {source.identifier} response is not bytes")
        chunks.append(chunk)
        total += len(chunk)
    payload = b"".join(chunks)
    verify_payload(source, payload)
    return payload


def download(source: SourceInput, timeout: float = 30.0, opener=None) -> bytes:
    request = Request(source.url, headers={"Accept-Encoding": "identity", "User-Agent": "WebBoxVM-F02"})
    opener = opener or build_opener(DenyRedirect())
    try:
        with opener.open(request, timeout=timeout) as response:
            return response_bytes(source, response)
    except ContractError:
        raise
    except HTTPError as error:
        if 300 <= error.code < 400:
            reject(f"input {source.identifier} redirect denied ({error.code})")
        reject(f"input {source.identifier} HTTP failure ({error.code})")
    except URLError as error:
        reject(f"input {source.identifier} network failure: {error.reason}")


def atomic_store(cache: ExternalCache, source: SourceInput, payload: bytes) -> Path:
    verify_payload(source, payload)
    target = cache.target(source)
    if target.is_symlink():
        reject(f"input {source.identifier} cache target is a symlink")
    if target.exists():
        verify_payload(source, target.read_bytes())
        return target
    target.parent.mkdir(parents=True, exist_ok=True)
    target = cache.target(source)
    if target.is_symlink():
        reject(f"input {source.identifier} cache target is a symlink")
    descriptor, temporary = tempfile.mkstemp(prefix=f".{source.identifier}-", suffix=".tmp", dir=target.parent)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    return target


def fetch_to_cache(cache: ExternalCache, source: SourceInput, timeout: float = 30.0, opener=None) -> tuple[Path, bool]:
    target = cache.target(source)
    if target.is_symlink():
        reject(f"input {source.identifier} cache target is a symlink")
    if target.exists():
        verify_payload(source, target.read_bytes())
        return target, True
    return atomic_store(cache, source, download(source, timeout, opener)), False
