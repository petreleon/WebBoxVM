"""Strict, bounded HTTPS JSON transport for immutable GitHub Git metadata."""
from __future__ import annotations

import json
import math
import re
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

API = "https://api.github.com/repos/KhronosGroup/VK-GL-CTS/git"
TAG = "vulkan-cts-1.4.6.2"
CHUNK, MAX_JSON = 64 * 1024, 2 * 1024 * 1024
HEX40 = re.compile(r"^[0-9a-f]{40}$")
PATH = re.compile(r"^/repos/KhronosGroup/VK-GL-CTS/git/(?:ref/tags/vulkan-cts-1\.4\.6\.2|(?:tags|commits|trees)/[0-9a-f]{40})$")


class GitHubTreeError(ValueError):
    """GitHub Git metadata was redirected, partial, malformed, or substituted."""


def reject(message: str) -> None:
    raise GitHubTreeError(message)


def no_duplicate_keys(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject("GitHub metadata JSON has a duplicate key")
        value[key] = item
    return value


class DenyRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        reject(f"GitHub metadata redirect denied ({code}) to {newurl}")


def sha1(value: object, label: str) -> str:
    if not isinstance(value, str) or not HEX40.fullmatch(value) or value == "0" * 40:
        reject(f"{label} is not a nonzero Git SHA-1")
    return value


def endpoint(kind: str, value: str, recursive: bool = False) -> str:
    if kind not in {"tags", "commits", "trees"} or recursive and kind != "trees":
        reject("GitHub metadata endpoint is unsupported")
    return f"{API}/{kind}/{sha1(value, 'GitHub URL SHA-1')}{'?recursive=1' if recursive else ''}"


def ref_endpoint(tag: str) -> str:
    if tag != TAG:
        reject("GitHub metadata tag ref is unsupported")
    return f"{API}/ref/tags/{tag}"


def safe_url(url: str) -> None:
    part = urlsplit(url)
    if (part.scheme != "https" or part.netloc != "api.github.com" or not PATH.fullmatch(part.path)
            or part.fragment or part.query not in ("", "recursive=1")
            or part.query and not part.path.startswith("/repos/KhronosGroup/VK-GL-CTS/git/trees/")):
        reject("GitHub metadata URL is not an exact HTTPS API endpoint")


def json_document(url: str, timeout: float, opener=None) -> dict[str, object]:
    safe_url(url)
    if not isinstance(timeout, (int, float)) or isinstance(timeout, bool) or not math.isfinite(timeout) or timeout <= 0:
        reject("GitHub metadata timeout is invalid")
    request = Request(url, headers={"Accept": "application/vnd.github+json", "Accept-Encoding": "identity", "User-Agent": "WebBoxVM-V2"})
    try:
        response = (opener or build_opener(DenyRedirect())).open(request, timeout=timeout)
    except HTTPError as error:
        reject(f"GitHub metadata HTTP failure ({error.code})")
    except (URLError, OSError) as error:
        reject(f"GitHub metadata network failure: {error}")
    try:
        headers, length = response.headers, response.headers.get("Content-Length")
        content_type, encoding = headers.get("Content-Type", ""), headers.get("Content-Encoding")
        if (response.geturl() != url or response.getcode() != 200 or encoding not in (None, "", "identity")
                or not isinstance(content_type, str) or content_type.split(";", 1)[0].strip().lower() != "application/json"
                or length is not None and (not length.isdecimal() or int(length) > MAX_JSON)):
            reject("GitHub metadata response identity, encoding, length, or type is invalid")
        body = bytearray()
        while chunk := response.read(CHUNK):
            if not isinstance(chunk, bytes) or len(body) + len(chunk) > MAX_JSON:
                reject("GitHub metadata JSON exceeds its byte limit")
            body.extend(chunk)
    finally:
        response.close()
    try:
        value = json.loads(body.decode("utf-8"), object_pairs_hook=no_duplicate_keys)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"GitHub metadata JSON cannot be parsed: {error}")
    if not isinstance(value, dict):
        reject("GitHub metadata JSON root is not an object")
    return value
