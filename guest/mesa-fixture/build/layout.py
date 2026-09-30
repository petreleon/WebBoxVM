"""Mesa25.3.6's stock Gallium packaging contract, verified in pinned Meson files."""
from pathlib import Path


def driver_layout(prefix, version):
    prefix = Path(prefix)
    libraries = sorted(path.name for path in (prefix / 'lib').glob('libgallium*.so'))
    expected = 'libgallium-' + version + '.so'
    aliases = list(prefix.rglob('*_dri.so'))
    icds = sorted((prefix / 'share/vulkan/icd.d').glob('*.json'))
    if libraries != [expected] or aliases or [path.name for path in icds] != ['virtio_icd.aarch64.json']:
        raise ValueError('stock Mesa driver layout differs: ' + repr((libraries, aliases, icds)))
    for path in prefix.rglob('*'):
        if any(name in path.name for name in ['swrast', 'lavapipe', 'lvp']):
            raise ValueError('software driver entered runtime closure')
    return {'drivers': ['virgl'], 'gallium_library': str(prefix / 'lib' / expected),
            'legacy_dri_aliases': [], 'icds': [str(path) for path in icds]}
