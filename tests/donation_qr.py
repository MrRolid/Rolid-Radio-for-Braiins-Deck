"""Run after smoke.py and layout_preview.py. Requires Pillow and zxing-cpp."""
from pathlib import Path
import re
from PIL import Image
import zxingcpp
root=Path(__file__).resolve().parents[1]
addresses=re.findall(r'"(bc1[^"]+)"',(root/'service/src/donation.rs').read_text())
assert len(addresses)==4
assert addresses==re.findall(r'"(bc1[^"]+)"',(root/'src/donation.rs').read_text())
chars='qpzry9x8gf2tvdw0s3jn54khce6mua7l';gen=[0x3b6a57b2,0x26508e6d,0x1ea119fa,0x3d4233dd,0x2a1462b3]
for i,address in enumerate(addresses):
 hrp,data=address.rsplit('1',1);chk=1
 for x in [ord(c)>>5 for c in hrp]+[0]+[ord(c)&31 for c in hrp]+[chars.index(c) for c in data]:
  top=chk>>25;chk=((chk&0x1ffffff)<<5)^x
  for j,g in enumerate(gen):
   if (top>>j)&1:chk^=g
 assert chk==1 and address.startswith('bc1q') and len(address)==42
 for width,height in [(317,238),(638,238),(638,480),(1280,480)]:
  r=zxingcpp.read_barcode(Image.open(f'/tmp/radio-preview-donation{i}-{width}-{height}.png'))
  assert r and r.text=='bitcoin:'+address,(i,width,height)
print('PASS address checksums, service/widget agreement and exact QR payload at all sizes')
