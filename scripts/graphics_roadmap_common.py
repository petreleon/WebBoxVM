"""Shared structural helpers for the graphics-roadmap checker and its tests."""

import re
from pathlib import Path
from urllib.parse import unquote

BOX = re.compile(r"^- \[([ x])\] (.+)$", re.M)
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
BLOCKED_BY = re.compile(r"external/[a-z0-9]+(?:-[a-z0-9]+)*")
RECEIPT_FIELDS = ("Revision", "Validation", "Result", "Artifacts", "Profile")
PLACEHOLDERS = ("pending", "todo", "tbd")


def field(body: str, name: str) -> str:
    match = re.search(rf"^{name}: (.+)$", body, re.M)
    return match.group(1).strip() if match else ""


def target(path: Path, link: str) -> Path | None:
    if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", link) or link.startswith("#"):
        return None
    return (path.parent / unquote(link.split("#", 1)[0])).resolve()


def checked(body: str) -> bool:
    boxes = BOX.findall(body)
    return bool(boxes) and all(mark == "x" for mark, _ in boxes)


def has_child_links(body: str) -> bool:
    return any(LINK.fullmatch(label) for _, label in BOX.findall(body))


def receipt_problems(
    path: Path, body: str, expected: str, missing: str, result: str, local: bool = False
) -> tuple[tuple[Path, str], ...]:
    evidence = LINK.search(field(body, "Evidence"))
    receipt = target(path, evidence.group(1)) if evidence else None
    if receipt is None or not receipt.is_file() or (local and receipt.parent != path.parent):
        return ((path, missing),)
    receipt_body = receipt.read_text()
    problems = [(receipt, f"missing concrete {name}") for name in RECEIPT_FIELDS
                if not (value := field(receipt_body, name)) or value.lower() in PLACEHOLDERS]
    if field(receipt_body, "Result") != expected:
        problems.append((receipt, result))
    return tuple(problems)
