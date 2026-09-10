"""HTTPS streaming with redirect, encoding, and identity checks."""

from __future__ import annotations

from urllib.error import HTTPError, URLError
from urllib.request import HTTPRedirectHandler, Request, build_opener

from vcts_cache_fs import CHUNK, reject


class DenyRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        reject(f"redirect denied ({code}) to {newurl}")


def chunks(url: str, timeout: float, expected: dict[str, object], opener=None):
    if not isinstance(url, str) or not url.startswith("https://") or timeout <= 0:
        reject("cache transport request is invalid")
    request = Request(url, headers={"Accept-Encoding": "identity", "User-Agent": "WebBoxVM-V2"})
    try:
        response = (opener or build_opener(DenyRedirect())).open(request, timeout=timeout)
    except HTTPError as error:
        reject(f"cache HTTP failure ({error.code})")
    except URLError as error:
        reject(f"cache network failure: {error.reason}")
    try:
        length = response.headers.get("Content-Length")
        if (response.geturl() != url or response.getcode() not in (None, 200)
                or response.headers.get("Content-Encoding") not in (None, "", "identity")
                or (length is not None and (not length.isdecimal() or int(length) != expected["bytes"]))):
            reject("cache response identity, encoding, or length is invalid")
        while chunk := response.read(CHUNK):
            yield chunk
    finally:
        response.close()
