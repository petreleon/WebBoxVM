"""Finite hard/soft module closure with canonical paths and explicit edge records."""

from pathlib import PurePosixPath
import re


def path_name(value):
    path = PurePosixPath(value)
    if (not value or value.startswith("/") or "\\" in value or "\0" in value
            or any(part in {"", ".", ".."} for part in value.split("/"))):
        raise ValueError("unsafe module dependency path")
    if not any(value.endswith(suffix) for suffix in (".ko", ".ko.xz", ".ko.gz", ".ko.zst")):
        raise ValueError("dependency is not a kernel module")
    return path.as_posix()


def module_name(path):
    name = PurePosixPath(path).name.split(".ko")[0].replace("-", "_")
    if not re.fullmatch(r"[A-Za-z0-9_]+", name):
        raise ValueError("malformed kernel module name")
    return name


def dependencies(text):
    rows = {}
    for line in text.splitlines():
        if not line:
            continue
        key, separator, values = line.partition(":")
        if not separator or ":" in values:
            raise ValueError("malformed modules.dep row")
        key = path_name(key)
        if key in rows:
            raise ValueError("duplicate modules.dep row")
        rows[key] = [path_name(value) for value in values.split()]
    if any(value not in rows for values in rows.values() for value in values):
        raise ValueError("hard dependency is missing from modules.dep")
    return rows


def soft_dependencies(text, wanted=None):
    rows = {}
    for line in text.splitlines():
        words = line.split()
        if not words or words[0].startswith("#"):
            continue
        if len(words) < 2 or words[0] != "softdep":
            raise ValueError("malformed modules.softdep row")
        name = soft_name(words[1])
        if wanted is not None and name not in wanted:
            continue
        if len(words) < 3:
            raise ValueError("malformed modules.softdep row")
        kind = None
        for word in words[2:]:
            if word in {"pre:", "post:"}:
                kind = word[:-1]
            elif kind is None:
                raise ValueError("soft dependency lacks pre/post classification")
            else:
                rows.setdefault(name, []).append((kind, soft_name(word)))
    return rows


def soft_name(value):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", value):
        raise ValueError("soft dependency must name a module, not a path")
    return value.replace("-", "_")


def closure(directory, target="virtio_gpu"):
    targets = (target,) if isinstance(target, str) else tuple(target)
    if not targets:
        raise ValueError("at least one module root is required")
    hard = dependencies((directory / "modules.dep").read_text())
    soft_text = (directory / "modules.softdep").read_text()
    names = {}
    for path in hard:
        name = module_name(path)
        if name in names:
            raise ValueError("ambiguous loadable module name")
        if (directory / path).is_symlink() or not (directory / path).is_file():
            raise ValueError("loadable dependency is not a regular file")
        names[name] = path
    builtins = {module_name(path_name(path)) for path in (directory / "modules.builtin").read_text().splitlines()}
    for name in targets:
        if name not in names:
            raise ValueError("requested module is missing: " + name)
    pending, selected, edges = [names[name] for name in targets], set(), []
    # Each path enters selected once; cycles do not expand the finite graph indefinitely.
    while pending:
        path = pending.pop()
        if path in selected:
            continue
        selected.add(path)
        for dependency in hard[path]:
            edges.append({"source": path, "kind": "hard", "target": dependency, "builtin": False})
            pending.append(dependency)
        own_name = module_name(path)
        for kind, name in soft_dependencies(soft_text, {own_name}).get(own_name, []):
            if name in names:
                dependency, builtin = names[name], False
                pending.append(dependency)
            elif name in builtins:
                dependency, builtin = name, True
            else:
                raise ValueError("soft dependency cannot be resolved: " + name)
            edges.append({"source": path, "kind": "soft-" + kind, "target": dependency, "builtin": builtin})
    edges.sort(key=lambda row: (row["source"], row["kind"], row["target"]))
    return sorted(selected), edges


def copy_closure(source, destination, selected):
    for path in selected:
        target = destination / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((source / path).read_bytes())
        target.chmod(0o644)
    for name in ("modules.builtin", "modules.builtin.modinfo"):
        (destination / name).write_bytes((source / name).read_bytes())
    wanted = {path.removesuffix(".xz").removesuffix(".gz").removesuffix(".zst") for path in selected}
    order = (source / "modules.order").read_text().splitlines()
    (destination / "modules.order").write_text("".join(path + "\n" for path in order if path in wanted))
