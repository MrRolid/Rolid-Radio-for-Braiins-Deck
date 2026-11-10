#!/usr/bin/env python3
"""Restore pinned SDK/vendor beside the project, without overwriting work."""
from pathlib import Path
import tarfile,tempfile,shutil,json
root=Path(__file__).resolve().parents[1];parent=root.parent
sdk=parent/'bmc-main';vendor=parent/'vendor';config=root/'.cargo/config.toml'
revision='b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af'
if config.exists():raise SystemExit('Existing .cargo/config.toml: inspect and merge manually; no files changed.')
if sdk.exists() or vendor.exists():raise SystemExit('Sibling bmc-main or vendor already exists. Use a fresh parent folder or inspect manually; no files changed.')
archive=root/'package/release/source-dependencies.tar.gz'
with tempfile.TemporaryDirectory(prefix='radio-deps-',dir=parent) as tmp:
 with tarfile.open(archive) as t:
  for m in t.getmembers():
   if Path(m.name).parts[0] not in ('bmc-main','vendor','.cargo'):raise SystemExit('Unexpected dependency archive member')
  t.extractall(tmp,members=[m for m in t.getmembers() if Path(m.name).parts[0] in ('bmc-main','vendor')],filter='data')
 p=Path(tmp)
 if (p/'bmc-main/SDK_REVISION').read_text().strip()!=revision:raise SystemExit('SDK revision mismatch')
 shutil.move(str(p/'bmc-main'),sdk);shutil.move(str(p/'vendor'),vendor)
config.parent.mkdir(exist_ok=True)
config.write_text('[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = '+json.dumps(str(vendor))+'\n')
print('Pinned SDK and vendor restored. Local Cargo configuration written.')
print('Install Rust 1.99.1 and both targets separately, then run sh tools/build-offline.sh.')
