"""Load the fixed shared engine without consuming ambient package aliases."""

import importlib.util
import sys
from pathlib import Path


def load(package: Path, namespace: str):
    if package.is_symlink() or not package.is_dir():
        raise ValueError("fixed inventory package must be a regular directory")
    for module_file in package.glob("*.py"):
        if module_file.is_symlink() or not module_file.is_file():
            raise ValueError("fixed inventory module must be a regular file")
    prefix = namespace + "."
    prior = {name: module for name, module in sys.modules.items() if name == namespace or name.startswith(prefix)}
    for name in prior:
        sys.modules.pop(name)
    try:
        spec = importlib.util.spec_from_file_location(namespace, package / "__init__.py",
                                                     submodule_search_locations=[str(package)])
        if spec is None or spec.loader is None:
            raise ValueError("cannot load fixed inventory package")
        module = importlib.util.module_from_spec(spec)
        sys.modules[namespace] = module
        spec.loader.exec_module(module)
        for name, loaded in tuple(sys.modules.items()):
            if name == namespace or name.startswith(prefix):
                actual = Path(getattr(loaded, "__file__", "")).resolve()
                if actual.parent != package.resolve():
                    raise ValueError("inventory module resolved outside the fixed package")
        return module
    finally:
        for name in tuple(sys.modules):
            if name == namespace or name.startswith(prefix):
                sys.modules.pop(name)
        sys.modules.update(prior)
