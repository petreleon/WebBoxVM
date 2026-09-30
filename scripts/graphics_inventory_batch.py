#!/usr/bin/env python3
"""Run the fixed inventory pipeline independently of cwd and ambient Python packages."""

import importlib.util
from pathlib import Path


def main():
    package = Path(__file__).resolve().parent / "graphics/inventories"
    bootstrap_file = package / "bootstrap.py"
    if bootstrap_file.is_symlink() or not bootstrap_file.is_file():
        raise ValueError("fixed inventory bootstrap must be a regular file")
    spec = importlib.util.spec_from_file_location("inventory_batch_bootstrap", bootstrap_file)
    bootstrap = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(bootstrap)
    api = bootstrap.load(package, "_fixed_inventory_batch")
    api.batch_main()


if __name__ == "__main__":
    main()
