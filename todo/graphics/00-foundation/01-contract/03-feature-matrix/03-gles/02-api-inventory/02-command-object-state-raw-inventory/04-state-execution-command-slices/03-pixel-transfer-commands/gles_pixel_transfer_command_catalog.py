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
ROUTE = 'pixel-transfer-commands'
RAW_PREFIX = 'gles32-pixel-transfer'
PRIMARY_PAGE = 426
PRIMARY_SECTION = '16.1'
FAMILIES = ({'anchor': {'kind': 'declaration',
             'physical_page': 426,
             'section': '16.1',
             'text': 'void ReadPixels( int x, int y, sizei width, sizei height, enum format, enum type, void '
                     '*data );'},
  'family_kind': 'declaration',
  'id': 'pixel-store-and-transfer',
  'reason': 'formal-declaration-only',
  'route': 'pixel-transfer-commands',
  'source_order': 19,
  'source_scope': ['8.4', '16']},)
WINDOWS = (('pixel-store-and-transfer',
  162,
  175,
  '8.4 Pixel Rectangles',
  '8.5 Texture Image Specification',
  1,
  'ed306cf6d642e219c4ab3818bf2e2fa6a251a60aac6260a3e8a5a7c6422d0094'),
 ('pixel-store-and-transfer',
  424,
  439,
  'Chapter 16',
  'Chapter 17',
  5,
  'b0db52a41ae13b92d4db1de0e38477349b2efe6ecdfc6f2f6e3ca07605314c9c'))
EXCLUSIONS = ()
TYPOGRAPHY = ()
INDEX_WITNESSES = ((589, 'PixelStorei'),
 (590, 'ReadBuffer'),
 (591, 'ReadPixels'),
 (590, 'ReadnPixels'),
 (572, 'BlitFramebuffer'),
 (575, 'CopyImageSubData'))
ROUTED_DECLARATIONS = ()
EXPANDED_COUNT = 6
BOUNDARY_SHA256 = '39e8d2116182782ffdc4311fecd7e1df7b240a0e33290332a567a2d10c475a37'

CONTRACT = SimpleNamespace(**{name: globals()[name] for name in (
    "PROFILE", "PAGES", "ROUTE", "RAW_PREFIX", "FAMILIES", "WINDOWS", "EXCLUSIONS", "TYPOGRAPHY",
    "EXPANDED_COUNT", "BOUNDARY_SHA256", "INDEX_WITNESSES", "ROUTED_DECLARATIONS", "RAW_ROOT")})
CATALOG = HELPER.DeclarationCatalog(CONTRACT)
facts, bound_family = CATALOG.facts, CATALOG.bound_family
raw_defaults, coverage = CATALOG.raw_defaults, CATALOG.coverage
