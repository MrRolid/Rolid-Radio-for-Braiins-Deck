"""Run against a RESTORED NAR root, not only the build staging directory."""
from pathlib import Path
import json,sys
root=Path(sys.argv[1])
base=root/'lib/bmc-widgets'
assert sorted(p.name for p in root.iterdir())==['lib']
assert sorted(p.name for p in (root/'lib').iterdir())==['bmc-widgets']
assert sorted(p.name for p in base.iterdir())==['rolid-radio']
w=base/'rolid-radio'
files={str(p.relative_to(w)) for p in w.rglob('*') if p.is_file()}
assert {'manifest.json','bin/rolid-radio','lib/wasm/rolid-radio.wasm'} <= files
assert all(f in {'manifest.json','bin/rolid-radio','lib/wasm/rolid-radio.wasm'} or f.startswith('lib/assets/v1/svg/') for f in files)
assets=list((w/'lib/assets/v1/svg').glob('*.asset'))
assert len(assets)==15, len(assets)
import subprocess
subprocess.run([str(Path(__file__).resolve().parents[1]/'asset-pack/target/debug/rolid-asset-pack'),'verify',str(w/'lib/wasm/rolid-radio.wasm')],check=True)
assert not any(p.is_symlink() for p in root.rglob('*'))
m=json.loads((w/'manifest.json').read_text())
assert m['binary']=='bin/rolid-radio'
assert m['version']==json.loads((Path(__file__).resolve().parents[1]/'manifest.json').read_text())['version']
assert (w/'lib/wasm/rolid-radio.wasm').read_bytes()[:8]==b'\0asm\1\0\0\0'
template=(Path(__file__).resolve().parents[2]/'bmc-main/bmc-upgrade/assets/wasm-widget-wrapper.sh').read_text()
assert (w/'bin/rolid-radio').read_text()==template.replace('@sdkMajor@','0').replace('@name@','rolid-radio')
for rel in files:
 assert bool((w/rel).stat().st_mode & 0o111)==(rel=='bin/rolid-radio'),rel
print('Restored NAR: structure, wrapper, permissions, manifest path, WASM header PASS')
