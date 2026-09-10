"""Closed-world local replay of descriptor-anchored live VCTS payloads."""

from __future__ import annotations

import hashlib
import os
import stat
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
if str(CACHE) not in sys.path:
    sys.path.insert(0, str(CACHE))
import vcts_cache_fs as fs
from vcts_live_cleanup import discard_payloads

StageError = fs.CacheError


def reject(message: str) -> None:
    fs.reject(message)


@dataclass(frozen=True)
class Payload:
    name: str
    url: str
    path: str
    revision: str
    blob_sha1: str
    sha256: str
    bytes: int

    def ledger_row(self) -> dict[str, object]:
        return {"path": self.path, "parent_path": None, "revision": self.revision,
                "blob_sha1": self.blob_sha1, "sha256": self.sha256, "bytes": self.bytes}


def _inspect(fd: int, row: Payload) -> None:
    state = os.fstat(fd)
    if (not stat.S_ISREG(state.st_mode) or state.st_uid != os.geteuid() or state.st_mode & 0o022
            or state.st_size != row.bytes or row.bytes < 1):
        reject("local replay payload is not private regular content of the expected size")
    sha256, blob = hashlib.sha256(), hashlib.sha1(f"blob {row.bytes}\0".encode())
    while chunk := os.read(fd, fs.CHUNK):
        sha256.update(chunk)
        blob.update(chunk)
    if os.fstat(fd).st_size != row.bytes or (sha256.hexdigest(), blob.hexdigest()) != (row.sha256, row.blob_sha1):
        reject("local replay payload no longer matches its captured identities")
    os.lseek(fd, 0, os.SEEK_SET)


class LocalResponse:
    def __init__(self, fd: int, payload: Payload):
        self.fd, self.payload, self.closed = fd, payload, False
        self.total, self.complete = 0, False
        self.headers = {"Content-Length": str(payload.bytes), "Content-Encoding": "identity"}

    def geturl(self):
        return self.payload.url

    def getcode(self):
        return 200

    def read(self, size=-1):
        if self.closed:
            return b""
        if size == -1:
            size = fs.CHUNK
        if type(size) is not int or size < 0 or size > fs.CHUNK:
            reject("local replay read is outside its bounded contract")
        data = os.read(self.fd, size)
        self.total += len(data)
        if not data:
            if self.total != self.payload.bytes:
                reject("local replay payload ended before its declared byte count")
            self.complete = True
        return data

    def close(self):
        if not self.closed:
            os.close(self.fd)
            self.closed = True


class LocalOpener:
    """A closed-world urllib opener: planned URLs only, once each, never network."""
    def __init__(self, directory_fd: int, payloads: tuple[Payload, ...]):
        if len(payloads) != 99 or len({row.url for row in payloads}) != 99:
            reject("local replay does not have the exact captured raw URL set")
        self.fd, self.rows, self.used, self.responses = directory_fd, {payloads[0].url: payloads[0]}, set(), []
        seen = set()
        for row in payloads[1:]:
            if row.sha256 not in seen:
                self.rows[row.url] = row
                seen.add(row.sha256)

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def open(self, request, timeout):
        url = getattr(request, "full_url", None)
        if self.fd < 0 or type(timeout) not in (int, float) or timeout <= 0 or url not in self.rows:
            reject("local replay request is not an expected immutable URL")
        if url in self.used:
            reject("local replay refuses a repeated payload request")
        payload, file_fd = self.rows[url], fs.file(self.fd, self.rows[url].name)
        try:
            _inspect(file_fd, payload)
        except BaseException:
            os.close(file_fd)
            raise
        self.used.add(url)
        response = LocalResponse(file_fd, payload)
        self.responses.append(response)
        return response

    def assert_complete(self):
        if set(self.rows) != self.used or any(not row.complete for row in self.responses):
            reject("local replay did not consume every captured payload exactly once")

    def close(self):
        for response in self.responses:
            response.close()
        self.responses.clear()
        if self.fd >= 0:
            os.close(self.fd)
            self.fd = -1


@dataclass
class LiveStage:
    directory: Path
    root: Payload
    members: tuple[Payload, ...]
    _fd: int
    _parent_fd: int
    _leaf: str

    @property
    def total_bytes(self) -> int:
        return self.root.bytes + sum(row.bytes for row in self.members)

    def replay(self) -> LocalOpener:
        if self._fd < 0:
            reject("live stage is closed")
        return LocalOpener(os.dup(self._fd), (self.root, *self.members))

    def close(self):
        if self._fd >= 0:
            os.close(self._fd)
            self._fd = -1
        if self._parent_fd >= 0:
            os.close(self._parent_fd)
            self._parent_fd = -1

    def discard(self):
        if self._fd < 0 or self._parent_fd < 0:
            reject("live stage is closed")
        directory_fd, parent_fd = self._fd, self._parent_fd
        self._fd = self._parent_fd = -1
        discard_payloads(directory_fd, parent_fd, self._leaf, tuple(row.name for row in (self.root, *self.members)))
