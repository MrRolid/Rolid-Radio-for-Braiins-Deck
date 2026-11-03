from pathlib import Path
import shutil,sys,subprocess
root = Path(__file__).resolve().parent
out = root / 'package/root'
if out.exists():
    shutil.rmtree(out)
w = out / 'lib/bmc-widgets/rolid-radio'
for part in ['bin', 'lib/wasm']:
    (w / part).mkdir(parents=True, exist_ok=True)
shutil.copyfile(root / (sys.argv[1] if len(sys.argv)>1 else 'manifest.json'), w / 'manifest.json')
subprocess.run([str(root/'asset-pack/target/debug/rolid-asset-pack'),str(root/'target/wasm32-unknown-unknown/release/rolid_radio.wasm'),str(w/'lib/wasm/rolid-radio.wasm'),str(w/'lib/assets'),str(root/'target/wasm32-unknown-unknown/release/deps')],check=True)
for temporary in (w / 'lib').glob('.bmc-assets-*'):
    shutil.rmtree(temporary)
template = (root.parent / 'bmc-main/bmc-upgrade/assets/wasm-widget-wrapper.sh').read_text()
launcher = w / 'bin/rolid-radio'
launcher.write_text(template.replace('@sdkMajor@', '0').replace('@name@', 'rolid-radio'))
launcher.chmod(0o755)
for p in [w / 'manifest.json', w / 'lib/wasm/rolid-radio.wasm']:
    p.chmod(0o644)
print(out)
