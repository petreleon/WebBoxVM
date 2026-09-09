"""Pinned, observer-only container plan for one Vulkan Docs replay."""

from __future__ import annotations

import hashlib
import shlex
import sys
from pathlib import Path

from vulkan_docs_observer_model import reject
from vulkan_docs_observer_events import producer_digest
from vulkan_docs_observer_normalize import tree
from vulkan_docs_observer_parse import canonical, digest_file

HERE = Path(__file__).resolve().parent
IDENTITY = HERE.parent.parent / "02-actual-closure-identity"
if str(IDENTITY) not in sys.path:
    sys.path.insert(0, str(IDENTITY))

from vulkan_docs_identity_build import ARGV, ENVIRONMENT, IMAGE, OUTPUT, TREE  # noqa: E402
from vulkan_docs_identity_contract import witness  # noqa: E402
from vulkan_docs_identity_members import DOCS_COMMIT  # noqa: E402

PATH = "/opt/venv/bin:/usr/local/bundle/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
OBSERVER = HERE / "observer"
OBSERVER_FILES = (
    "observer/input_trace.c", "observer/asciidoctor_include_hook.rb", "vulkan_docs_observer_model.py",
    "vulkan_docs_observer_parse.py", "vulkan_docs_observer_events.py", "vulkan_docs_observer_normalize.py",
    "vulkan_docs_observer_plan.py", "vulkan_docs_observer_contract.py", "vulkan_docs_observer_run.py",
)
SOURCE_TREE = (1325, 28697318, "99cfd3132413891764f98f246567d0ce0344f2f0bb5499bfbc7fc47646fa122b")


def observer_digest() -> str:
    rows = [{"selector": name, "sha256": digest_file(HERE / name, "observer source")} for name in OBSERVER_FILES]
    return canonical(rows, "webboxvm-graphics-vulkan-docs-observer-sources-v1")


def environment(include_hook: bool = False) -> list[str]:
    values = {name: value for name, value in ENVIRONMENT}
    values["HOME"] = "/tmp"
    values["PATH"] = PATH
    if include_hook:
        values["RUBYOPT"] = "-r/observer/asciidoctor_include_hook.rb"
        values["WEBBOXVM_INCLUDE_TRACE"] = "/work/observer/resolved-includes.jsonl"
    return [f"{name}={value}" for name, value in sorted(values.items())]


def docker_prefix(source: Path, work: Path, include_hook: bool) -> list[str]:
    if not source.is_absolute() or not work.is_absolute():
        reject("observer mounts must be absolute")
    command = ["docker", "run", "--rm", "--network", "none", "--platform", "linux/amd64", "--user", "501:20"]
    for value in environment(include_hook):
        command.extend(("-e", value))
    command.extend(("-v", f"{source}:/vulkan:ro", "-v", f"{work}:/work:rw", "-v", f"{OBSERVER}:/observer:ro"))
    return command + ["-w", "/vulkan", "--entrypoint", "sh", IMAGE, "-c"]


def dry_run_command(source: Path, work: Path) -> list[str]:
    argv = (ARGV[0], "-n", *ARGV[1:])
    return docker_prefix(source, work, False) + [shlex.join(argv)]


def build_command(source: Path, work: Path) -> list[str]:
    compiler = "gcc -shared -fPIC -O2 -Wall -Werror -ldl -o /work/observer/libinputtrace.so /observer/input_trace.c"
    build = "LD_PRELOAD=/work/observer/libinputtrace.so WEBBOXVM_TRACE=/work/observer/io-events.jsonl " + shlex.join(ARGV)
    return docker_prefix(source, work, True) + [f"{compiler} && {build}"]


def output_identity(generated: Path) -> tuple[int, int, str, str]:
    count, total, tree_digest = tree(generated)
    html = generated / OUTPUT["selector"]
    html_digest = digest_file(html, "observer primary HTML")
    if (count, total, tree_digest) != TREE or html_digest != OUTPUT["sha256"]:
        reject("observer replay does not match the pinned output witness")
    return count, total, tree_digest, html_digest


def witness_digest() -> str:
    return witness().digest
