"""Reviewed source-audit locations and isolated dependency loaders for the source map."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE_MAP = HERE / "source_map.json"
AUDITS = {"opengl-4.6-core": ROOT / "01-opengl-input-audit/candidates.json",
          "gles-3.2": ROOT / "02-gles-input-audit/candidates.json",
          "vulkan-1.4-core": ROOT / "03-vulkan-input-audit/candidates.json"}
GLES_CLOSURE = ROOT / "02-gles-input-audit/cts_closure.json"
GLES_CONFIGURATIONS = ROOT / "02-gles-input-audit/cts_configurations.json"
VULKAN_INCLUDES = ROOT / "03-vulkan-input-audit/spec_includes.json"
VULKAN_REFERENCES = ROOT / "03-vulkan-input-audit/mustpass_references.json"


def reviewed_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module


CANDIDATES = reviewed_module("f024_source_map_candidates", ROOT / "candidate_contract.py")
PROFILE = reviewed_module("f024_source_map_profile", ROOT.parent.parent / "03-feature-matrix/01-profile-scope/profile_contract.py")
GLES = reviewed_module("f024_source_map_gles", HERE.parent / "closure_shape_contract.py")
INCLUDES = reviewed_module("f024_source_map_includes", ROOT / "03-vulkan-input-audit/spec_include_contract.py")
REFERENCES = reviewed_module("f024_source_map_references", ROOT / "03-vulkan-input-audit/mustpass_reference_contract.py")
EXPECTED = tuple((profile, role, identifier) for profile, role, identifier, _ in PROFILE.EXPECTED_REQUIREMENTS)
