"""Create a source-only release ZIP from an explicit list of project paths."""
from pathlib import Path
import json
import zipfile

root = Path(__file__).resolve().parents[1]
version = json.loads((root / 'manifest.json').read_text())['version']
out = root / 'package/release' / f'rolid-radio-{version}-source.zip'
files = [root / name for name in (
    '.gitignore', 'AGENTS.md', 'CONTRIBUTING.md', 'Cargo.toml', 'Cargo.lock',
    'LICENSE', 'README.md', 'RELEASE_NOTES.md', 'TESTING.md',
    'THIRD_PARTY_NOTICES.md', 'manifest.json', 'package.py', 'stations.json',
    'translations.json', 'package/release/.gitkeep',
)]
for name in ('.github', 'LICENSES', 'asset-pack', 'assets', 'demo-network',
             'docs', 'manifest-check', 'service', 'service-package', 'src',
             'tests', 'tools'):
    for p in (root / name).rglob('*'):
        rel = p.relative_to(root)
        if any(part in {'target', '__pycache__', 'node_modules', '.venv'}
               for part in rel.parts):
            continue
        if p.name.startswith('.env') or p.suffix in {'.log', '.pyc', '.part'}:
            continue
        if rel == Path('service-package/rolid-radio-service'):
            continue
        if p.is_symlink():
            raise ValueError(f'Unexpected source symlink: {rel}')
        if p.is_file():
            files.append(p)
out.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
    for p in sorted(files):
        archive.write(p, Path('rolid-radio') / p.relative_to(root))
print(out)
