"""Independent NAR reader; validates writer against the prior nix-store release."""
from pathlib import Path
import struct,io,sys,importlib.util,tempfile
spec=importlib.util.spec_from_file_location('nar',Path(__file__).parents[1]/'tools/nar.py');nar=importlib.util.module_from_spec(spec);spec.loader.exec_module(nar)
def restore(source,dest):
 f=open(source,'rb')
 def token():
  raw=f.read(8);assert len(raw)==8;n=struct.unpack('<Q',raw)[0];v=f.read(n);assert len(v)==n;assert f.read((-n)%8)==b'\0'*((-n)%8);return v
 def expect(v):assert token()==v
 def node(p):
  expect(b'(');expect(b'type');typ=token()
  if typ==b'directory':
   p.mkdir(parents=True,exist_ok=True);prev=b''
   while (t:=token())!=b')':
    assert t==b'entry';expect(b'(');expect(b'name');name=token();assert name>prev and b'/' not in name and name not in [b'.',b'..'];prev=name;expect(b'node');node(p/name.decode());expect(b')')
  else:
   assert typ==b'regular';t=token();exe=t==b'executable'
   if exe:expect(b'');t=token()
   assert t==b'contents';p.write_bytes(token());p.chmod(0o755 if exe else 0o644);expect(b')')
 expect(b'nix-archive-1');node(Path(dest));assert not f.read();f.close()
if __name__=='__main__':
 source=Path(sys.argv[1]);dest=Path(sys.argv[2]);restore(source,dest);out=io.BytesIO();nar.dump(dest,out);assert out.getvalue()==source.read_bytes();print('PASS exact NAR round trip:',source.name)
