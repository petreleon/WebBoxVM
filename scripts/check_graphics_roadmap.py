#!/usr/bin/env python3
"""Check nested graphics tasks, dependencies, receipts, links and line limits."""
import re
import sys
from pathlib import Path
from urllib.parse import unquote
REPO = Path(__file__).resolve().parents[1]
ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else REPO / "todo/graphics"
BOX = re.compile(r"^- \[([ x])\] (.+)$", re.M)
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
errors, pages, tasks = [], {}, {}
def fail(path, message):
    errors.append(f"{path}: {message}")
def field(body, name):
    match = re.search(rf"^{name}: (.+)$", body, re.M)
    return match.group(1).strip() if match else ""
def target(path, link):
    if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", link) or link.startswith("#"):
        return None
    return (path.parent / unquote(link.split("#", 1)[0])).resolve()
def checked(path):
    boxes = BOX.findall(pages.get(path, ""))
    return bool(boxes) and all(mark == "x" for mark, _ in boxes)
if not ROOT.is_dir():
    sys.exit(f"Roadmap directory is missing: {ROOT}")
for path in sorted(ROOT.rglob("*")):
    if not path.is_file():
        continue
    if "__pycache__" in path.parts or path.suffix == ".pyc":
        continue
    try:
        body = path.read_text()
    except (UnicodeError, OSError) as error:
        fail(path, f"cannot read maintained roadmap file: {error}")
        continue
    if len(body.splitlines()) > 180:
        fail(path, "exceeds 180 physical lines")
    if path.suffix != ".md":
        continue
    pages[path.resolve()] = body
    prose = re.sub(r"```.*?```|`[^`\n]+`", "", body, flags=re.S)
    for link in LINK.findall(prose):
        dest = target(path, link)
        if dest is not None and not dest.exists():
            fail(path, f"broken link: {link}")
    boxes = BOX.findall(body)
    limit = 8 if path == ROOT / "README.md" else 6
    if len(boxes) > limit:
        fail(path, f"has {len(boxes)} checkboxes; split into subfolders (max {limit})")
    ident = field(body, "Task")
    if not ident:
        continue
    if not re.fullmatch(r"[A-Z]+\d+(?:\.\d+)*", ident):
        fail(path, f"invalid task ID: {ident}")
    if ident in tasks:
        fail(path, f"duplicate task ID: {ident}")
    deps = field(body, "Depends")
    if not deps:
        fail(path, "missing Depends field; use none for no dependencies")
    tasks[ident] = (path.resolve(), [] if deps == "none" else deps.split(", "))
    for heading in ("## Outcome", "## Starting points", "## Checklist", "## Verification"):
        if heading not in body:
            fail(path, f"missing {heading}")
    if not boxes:
        fail(path, "task has no checkboxes")
statuses = {}
successors = {}
for path, body in pages.items():
    if path.name != "README.md" and not field(body, "Task"):
        continue
    statuses[path] = field(body, "Status") or "active"
    successors[path] = field(body, "Superseded-by")
    if statuses[path] not in ("active", "superseded"):
        fail(path, f"invalid Status: {statuses[path]}")
def superseded(path):
    if statuses.get(path) == "superseded":
        return True
    directory = path.parent
    while True:
        if statuses.get(directory / "README.md") == "superseded":
            return True
        if directory == ROOT:
            return False
        directory = directory.parent
passing = {}
checking = set()
def complete(path):
    if path in passing:
        return passing[path]
    if superseded(path) or not checked(path) or path in checking:
        return False
    checking.add(path)
    result = all(not (match := LINK.fullmatch(label)) or
                 (dest := target(path, match.group(1))) in pages and complete(dest)
                 for _, label in BOX.findall(pages.get(path, "")))
    checking.remove(path)
    passing[path] = result
    return result
for path, status in statuses.items():
    if status != "superseded":
        continue
    successor = successors[path]
    if not successor:
        fail(path, "superseded task needs Superseded-by")
    elif successor not in tasks:
        fail(path, f"unknown superseding task {successor}")
    elif superseded(tasks[successor][0]):
        fail(path, "Superseded-by must reference an active task")
for ident, (path, _) in tasks.items():
    if not checked(path):
        continue
    evidence = LINK.search(field(pages[path], "Evidence"))
    receipt = target(path, evidence.group(1)) if evidence else None
    if receipt is None or not receipt.is_file():
        fail(path, "completed task needs a local evidence receipt")
        continue
    receipt_body = receipt.read_text()
    for name in ("Revision", "Validation", "Result", "Artifacts", "Profile"):
        value = field(receipt_body, name)
        if not value or value.lower() in ("pending", "todo", "tbd"):
            fail(receipt, f"missing concrete {name}")
    if field(receipt_body, "Result") != "PASS":
        fail(receipt, "completed checklist requires Result: PASS")
for ident, (path, deps) in tasks.items():
    for dep in deps:
        if dep not in tasks:
            fail(path, f"unknown dependency {dep}")
        elif tasks[dep][0].parent in path.parents:
            fail(path, f"task depends on its own ancestor {dep}")
        elif not superseded(path) and superseded(tasks[dep][0]):
            fail(path, f"active task depends on superseded {dep}")
        elif complete(path) and not complete(tasks[dep][0]):
            fail(path, f"completed task depends on incomplete {dep}")
visited = set()
active = []
def visit(ident):
    if ident in active:
        fail(tasks[ident][0], "dependency cycle: " + " -> ".join(active + [ident]))
        return
    if ident in visited or ident not in tasks:
        return
    active.append(ident)
    for dep in tasks[ident][1]:
        visit(dep)
    active.pop()
    visited.add(ident)
for ident in tasks:
    visit(ident)
linked_lists = set()
for path, body in pages.items():
    if path.name != "README.md":
        continue
    for mark, label in BOX.findall(body):
        link = LINK.fullmatch(label)
        if not link:
            continue
        dest = target(path, link.group(1))
        if dest is None or dest not in pages:
            fail(path, "child checkbox must link to a local roadmap list")
            continue
        if dest.parent.parent != path.parent or dest.name != "README.md":
            fail(path, "child list must live in an immediate subfolder")
        linked_lists.add(dest)
        if not superseded(path) and (mark == "x") != (complete(dest) or superseded(dest)):
            fail(path, f"checkbox does not match child completion or supersession: {link.group(1)}")
for path in pages:
    if path.name == "README.md" and path != ROOT / "README.md" and path not in linked_lists:
        fail(path, "list is not linked from a parent checkbox")
if not tasks:
    fail(ROOT, "no task leaves found")
if errors:
    print("\n".join(errors), file=sys.stderr)
    sys.exit(1)
ready = [ident for ident, (path, deps) in tasks.items()
         if not superseded(path) and not complete(path) and all(complete(tasks[dep][0]) for dep in deps)
         and not any(LINK.fullmatch(label) for _, label in BOX.findall(pages[path]))]
done = sum(complete(path) for path, _ in tasks.values())
obsolete = sum(superseded(path) for path, _ in tasks.values())
print(f"PASS: {len(pages)} documents, {len(tasks)} tasks, {done} PASS-complete, {obsolete} superseded; links/dependencies/limits valid")
print("Ready: " + (", ".join(ready) if ready else "none"))
