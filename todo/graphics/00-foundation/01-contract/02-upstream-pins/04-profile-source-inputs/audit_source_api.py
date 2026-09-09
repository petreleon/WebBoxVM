#!/usr/bin/env python3
"""Exact-path loader for the reviewed F02 source-policy API."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INVENTORY_PATH = HERE.parent / "01-input-inventory/inventory_layout.py"
SOURCE_MODEL_PATH = HERE.parent / "02-fetch-verifier/01-fetch-contract/source_model.py"


def load_module(name: str, path: Path):
    """Load one reviewed sibling without accepting a basename-cache collision."""
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(name, None)
        raise
    return module


def source_api():
    """Return the F02 policy types while preserving an unrelated bare import."""
    original = sys.modules.get("inventory_layout")
    layout = load_module("f024_inventory_layout", INVENTORY_PATH)
    sys.modules["inventory_layout"] = layout
    try:
        model = load_module("f024_source_model", SOURCE_MODEL_PATH)
    finally:
        if original is None:
            sys.modules.pop("inventory_layout", None)
        else:
            sys.modules["inventory_layout"] = original
    return layout.FAMILIES, layout.load_inventory, model.ContractError, model.SourceInput
