"""Test-only in-memory transport fixtures; these never open a network socket."""

from __future__ import annotations

from source_model import SourceInput

PAYLOAD = b"F02.2.2 local transport payload\n"
PAYLOAD_BYTES = 32
PAYLOAD_SHA256 = "985b6e3e7172ecb3a4d72963dd3c84cbaf36ab6ae1efe0877891c15dc61d75b8"
REVISION = "c" * 40


def entry(**changes) -> dict[str, object]:
    value = {
        "id": "fixture-transport",
        "source_family": "fixture",
        "immutable_url": f"https://raw.githubusercontent.com/example/fixture/{REVISION}/payload.bin",
        "revision": REVISION,
        "sha256": PAYLOAD_SHA256,
        "bytes": PAYLOAD_BYTES,
        "license": "fixture",
        "local_cache": f"webboxvm-graphics/f02/fixture-transport/{PAYLOAD_SHA256}.source",
        "generated_code_role": "fixture",
        "provenance": "https://example.invalid/fixture",
    }
    value.update(changes)
    return value


def source(**changes) -> SourceInput:
    return SourceInput.from_manifest(entry(**changes))


class FakeResponse:
    """A context-managed byte stream with explicit final URL and status metadata."""

    def __init__(self, payload: object, final_url: str, status: int = 200) -> None:
        self.payload, self.final_url, self.status, self.offset = payload, final_url, status, 0

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, *_unused) -> bool:
        return False

    def geturl(self) -> str:
        return self.final_url

    def getcode(self) -> int:
        return self.status

    def read(self, size: int) -> object:
        chunk = self.payload[self.offset:self.offset + size]
        self.offset += len(chunk)
        return chunk


class FakeOpener:
    """Return one response or raise one injected exception; record every attempted request."""

    def __init__(self, outcome: object) -> None:
        self.outcome = outcome
        self.requests: list[tuple[str, float]] = []

    def open(self, request, timeout: float):
        self.requests.append((request.full_url, timeout))
        if isinstance(self.outcome, BaseException):
            raise self.outcome
        return self.outcome
