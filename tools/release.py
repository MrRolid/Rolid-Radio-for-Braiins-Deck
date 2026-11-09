from pathlib import Path
import subprocess,tarfile,hashlib,json
r=Path(__file__).resolve().parents[1];version=json.loads((r/'manifest.json').read_text())['version'];out=r/'package/release';out.mkdir(parents=True,exist_ok=True)
# Keep old releases elsewhere: one multilingual NAR per release directory.
for p in out.glob('*.nar'):p.unlink()
for p in out.glob('rolid-radio-service-*.tar.gz'):p.unlink()
with (out/f'rolid-radio-{version}.nar').open('wb') as f:subprocess.run([__import__('sys').executable,str(r/'tools/nar.py'),str(r/'package/root')],stdout=f,check=True)
with tarfile.open(out/f'rolid-radio-service-{version}.tar.gz','w:gz') as t:
 for n in ['install.sh','rolid-radio.init','clock-audio-bridge']:t.add(r/'service-package'/n,arcname=n)
 t.add(r/'service/target/armv7-unknown-linux-musleabihf/release/rolid-radio-service',arcname='rolid-radio-service')
(out/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in sorted(out.iterdir()) if p.is_file() and not p.name.startswith('.') and p.name!='SHA256SUMS'))
