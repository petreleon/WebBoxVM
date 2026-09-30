"""Complete finite formal-declaration windows from the exact GLES 3.2 PDF."""

import importlib.util
from pathlib import Path
from types import SimpleNamespace

HELPER_PATH = Path(__file__).resolve().parents[10] / "scripts/graphics/inventories/declarations.py"
if HELPER_PATH.is_symlink() or not HELPER_PATH.is_file():
    raise ValueError("fixed declaration helper must be a regular file")
SPEC = importlib.util.spec_from_file_location("webboxvm_state_declarations", HELPER_PATH)
if SPEC is None or SPEC.loader is None:
    raise ValueError("cannot load fixed declaration helper")
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)
CatalogError = HELPER.CatalogError
ENTRY_FIELDS, DEFAULT_FIELDS = HELPER.ENTRY_FIELDS, HELPER.DEFAULT_FIELDS
RAW_ROOT = Path(__file__).resolve().parents[2]
PROFILE, PAGES, USES_GRAMMAR_DOCUMENT, FRAGMENTED = "gles-3.2", 601, True, True
ROUTE = 'draw-raster-commands'
RAW_PREFIX = 'gles32-draw-raster'
PRIMARY_PAGE = 296
PRIMARY_SECTION = '10.5'
FAMILIES = ({'anchor': {'kind': 'declaration',
             'physical_page': 296,
             'section': '10.5',
             'text': 'void DrawArrays( enum mode, int first, sizei count );'},
  'family_kind': 'declaration',
  'id': 'draw',
  'reason': 'formal-declaration-only',
  'route': 'draw-raster-commands',
  'source_order': 23,
  'source_scope': ['10.5']},
 {'anchor': {'kind': 'heading',
             'physical_page': 293,
             'section': '10.3.9',
             'text': '10.3.9 Indirect Commands in Buffer Objects'},
  'family_kind': 'declaration',
  'id': 'indirect-draw',
  'reason': 'formal-declaration-only',
  'route': 'draw-raster-commands',
  'source_order': 24,
  'source_scope': ['10.3.9']},
 {'anchor': {'kind': 'declaration',
             'physical_page': 418,
             'section': '15.2.3',
             'text': 'void Clear( bitfield buf );'},
  'family_kind': 'declaration',
  'id': 'raster-and-framebuffer',
  'reason': 'formal-declaration-only',
  'route': 'draw-raster-commands',
  'source_order': 28,
  'source_scope': ['13', '15']},
 {'anchor': {'kind': 'declaration',
             'physical_page': 439,
             'section': '17',
             'text': 'void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );'},
  'family_kind': 'declaration',
  'id': 'compute',
  'reason': 'formal-declaration-only',
  'route': 'draw-raster-commands',
  'source_order': 30,
  'source_scope': ['17']})
WINDOWS = (('indirect-draw',
  293,
  294,
  '10.3.9 Indirect Commands in Buffer Objects',
  '10.4 Vertex Array Objects',
  0,
  '4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945'),
 ('draw',
  295,
  302,
  '10.5 Drawing Commands Using Vertex Arrays',
  '10.6 Vertex Array and Vertex Array Object Queries',
  10,
  'e6c5c4a1580da857ed61bc15a9a48ae063f400078f5ab3bb147a19354e03e37f'),
 ('raster-and-framebuffer',
  368,
  389,
  'Chapter 13',
  'Chapter 14',
  10,
  '41bc913998ebb0160e5a9fae02b577adb0bbac808a2ab64e2642bb7fa526a262'),
 ('raster-and-framebuffer',
  397,
  424,
  'Chapter 15',
  'Chapter 16',
  31,
  '6a32486830d13aa0f56fafcf02710daf0645991cfc461834deae76f0cd173c69'),
 ('compute',
  439,
  442,
  'Chapter 17',
  'Chapter 18',
  2,
  '74841d3ae30972937b1871b323a06b51d754107643552875138277a02aa30e44'))
EXCLUSIONS = ((295, 'DrawArraysOneInstance', 296, 'does not exist in the GL'),
 (298, 'DrawElementsOneInstance', 298, 'does not exist in the GL, but is used to describe functionality'))
TYPOGRAPHY = ()
INDEX_WITNESSES = ((576, 'DrawArrays'),
 (577, 'DrawArraysInstanced'),
 (577, 'DrawArraysIndirect'),
 (577, 'DrawElements'),
 (577, 'DrawElementsInstanced'),
 (577, 'DrawRangeElements'),
 (577, 'DrawElementsBaseVertex'),
 (577, 'DrawRangeElementsBaseVertex'),
 (577, 'DrawElementsInstancedBaseVertex'),
 (577, 'DrawElementsIndirect'),
 (590, 'PrimitiveBoundingBox'),
 (580, 'GetMultisamplefv'),
 (588, 'MinSampleShading'),
 (584, 'LineWidth'),
 (579, 'FrontFace'),
 (575, 'CullFace'),
 (590, 'PolygonOffset'),
 (593, 'Scissor'),
 (592, 'SampleCoverage'),
 (592, 'SampleMaski'),
 (594, 'StencilFunc'),
 (594, 'StencilFuncSeparate'),
 (594, 'StencilOp'),
 (594, 'StencilOpSeparate'),
 (576, 'DepthFunc'),
 (577, 'Enablei'),
 (576, 'Disablei'),
 (572, 'BlendEquation'),
 (572, 'BlendEquationSeparate'),
 (572, 'BlendEquationi'),
 (572, 'BlendEquationSeparatei'),
 (572, 'BlendFunc'),
 (572, 'BlendFuncSeparate'),
 (572, 'BlendFunci'),
 (572, 'BlendFuncSeparatei'),
 (572, 'BlendBarrier'),
 (572, 'BlendColor'),
 (577, 'DrawBuffers'),
 (573, 'ColorMask'),
 (573, 'ColorMaski'),
 (576, 'DepthMask'),
 (594, 'StencilMask'),
 (594, 'StencilMaskSeparate'),
 (573, 'Clear'),
 (573, 'ClearColor'),
 (573, 'ClearDepthf'),
 (573, 'ClearStencil'),
 (573, 'ClearBuffer{if ui}v'),
 (573, 'ClearBufferfi'),
 (583, 'InvalidateSubFramebuffer'),
 (583, 'InvalidateFramebuffer'),
 (576, 'DispatchCompute'),
 (576, 'DispatchComputeIndirect'))
ROUTED_DECLARATIONS = ()
EXPANDED_COUNT = 55
BOUNDARY_SHA256 = '0ff8a8cb1b0ccc02c36297e235eb394c3b8b88b00648feb0bda02f5a25b462f7'

CONTRACT = SimpleNamespace(**{name: globals()[name] for name in (
    "PROFILE", "PAGES", "ROUTE", "RAW_PREFIX", "FAMILIES", "WINDOWS", "EXCLUSIONS", "TYPOGRAPHY",
    "EXPANDED_COUNT", "BOUNDARY_SHA256", "INDEX_WITNESSES", "ROUTED_DECLARATIONS", "RAW_ROOT")})
CATALOG = HELPER.DeclarationCatalog(CONTRACT)
facts, bound_family = CATALOG.facts, CATALOG.bound_family
raw_defaults, coverage = CATALOG.raw_defaults, CATALOG.coverage
