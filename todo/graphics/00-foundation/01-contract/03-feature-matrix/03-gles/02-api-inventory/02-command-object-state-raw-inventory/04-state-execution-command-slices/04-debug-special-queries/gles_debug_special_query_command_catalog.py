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
ROUTE = 'debug-special-query-commands'
RAW_PREFIX = 'gles32-debug-special-query'
PRIMARY_PAGE = 445
PRIMARY_SECTION = '18.2'
FAMILIES = ({'anchor': {'kind': 'declaration',
             'physical_page': 445,
             'section': '18.2',
             'text': 'void DebugMessageCallback( DEBUGPROC callback, const void *userParam );'},
  'family_kind': 'declaration',
  'id': 'debug',
  'reason': 'formal-declaration-only',
  'route': 'debug-special-query-commands',
  'source_order': 31,
  'source_scope': ['18']},
 {'anchor': {'kind': 'declaration',
             'physical_page': 455,
             'section': '19.1',
             'text': 'void Hint( enum target, enum hint );'},
  'family_kind': 'declaration',
  'id': 'special',
  'reason': 'formal-declaration-only',
  'route': 'debug-special-query-commands',
  'source_order': 32,
  'source_scope': ['19']},
 {'anchor': {'kind': 'declaration',
             'physical_page': 457,
             'section': '20.1',
             'text': 'void GetBooleanv( enum pname, boolean *data );'},
  'family_kind': 'declaration',
  'id': 'context-queries',
  'reason': 'formal-declaration-only',
  'route': 'debug-special-query-commands',
  'source_order': 33,
  'source_scope': ['20']})
WINDOWS = (('debug',
  442,
  455,
  'Chapter 18',
  'Chapter 19',
  10,
  '24258bf4599017dfb215fc2252d5e4ca48530454efe72122192294079d968d4f'),
 ('special',
  455,
  457,
  'Chapter 19',
  'Chapter 20',
  1,
  '454db383943b0abfac0ed9b5fdaa86dbbadfa327a03ef9332357aa785a1f18cc'),
 ('context-queries',
  457,
  464,
  'Chapter 20',
  'Chapter 21',
  13,
  '6eb126460ebb4b2be374891d89381780e03e644845d945bc8012088a91d6a4c6'))
EXCLUSIONS = ()
TYPOGRAPHY = ((458, '20.1', 'GetBooleani v', 'GetBooleani_v'),
 (458, '20.1', 'GetIntegeri v', 'GetIntegeri_v'),
 (458, '20.1', 'GetInteger64i v', 'GetInteger64i_v'))
INDEX_WITNESSES = ((575, 'DebugMessageCallback'),
 (575, 'DebugMessageControl'),
 (575, 'DebugMessageInsert'),
 (590, 'PushDebugGroup'),
 (590, 'PopDebugGroup'),
 (589, 'ObjectLabel'),
 (589, 'ObjectPtrLabel'),
 (580, 'GetDebugMessageLog'),
 (580, 'GetObjectLabel'),
 (580, 'GetObjectPtrLabel'),
 (582, 'Hint'),
 (579, 'GetBooleanv'),
 (580, 'GetIntegerv'),
 (580, 'GetInteger64v'),
 (580, 'GetFloatv'),
 (579, 'GetBooleani v'),
 (580, 'GetIntegeri v'),
 (580, 'GetInteger64i v'),
 (583, 'IsEnabled'),
 (583, 'IsEnabledi'),
 (580, 'GetPointerv'),
 (581, 'GetString'),
 (581, 'GetStringi'),
 (580, 'GetInternalformativ'))
ROUTED_DECLARATIONS = (('03-object-resource-command-slices/01-generic-sync-query/gles_generic_sync_query_raw_inventory.json',
  '5c48f04b81f414e4d703465bc3ab33fe46bc178849d64819a2773edaf9df3cfa',
  'F03.3.2.2.3.1',
  'ca43e662ada91a2c72a0265e500f2055dbec09da30defcd432ec3ced04815013',
  'glGetGraphicsResetStatus',
  'enum GetGraphicsResetStatus( void );',
  34,
  '2.3'),)
EXPANDED_COUNT = 24
BOUNDARY_SHA256 = '3766d30aefe770de343371150d119c0c32a812fedba758e8a49313e2873eb7aa'

CONTRACT = SimpleNamespace(**{name: globals()[name] for name in (
    "PROFILE", "PAGES", "ROUTE", "RAW_PREFIX", "FAMILIES", "WINDOWS", "EXCLUSIONS", "TYPOGRAPHY",
    "EXPANDED_COUNT", "BOUNDARY_SHA256", "INDEX_WITNESSES", "ROUTED_DECLARATIONS", "RAW_ROOT")})
CATALOG = HELPER.DeclarationCatalog(CONTRACT)
facts, bound_family = CATALOG.facts, CATALOG.bound_family
raw_defaults, coverage = CATALOG.raw_defaults, CATALOG.coverage
