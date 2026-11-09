"""Deterministic Nix archive writer for a widget directory (no symlinks)."""
import os,struct,sys
from pathlib import Path

def dump(root,out):
 def string(s):
  if isinstance(s,str):s=s.encode()
  out.write(struct.pack('<Q',len(s)));out.write(s);out.write(b'\0'*((-len(s))%8))
 def node(p):
  if p.is_symlink():raise ValueError('Symlinks are not supported')
  string('(');string('type')
  if p.is_dir():
   string('directory')
   for child in sorted(p.iterdir(),key=lambda p:os.fsencode(p.name)):
    for v in ['entry','(','name',child.name,'node']:string(v)
    node(child);string(')')
  elif p.is_file():
   string('regular')
   if p.stat().st_mode&0o100:string('executable');string('')
   string('contents');string(p.read_bytes())
  else:raise ValueError('Unsupported file type')
  string(')')
 string('nix-archive-1');node(Path(root))
if __name__=='__main__':dump(sys.argv[1],sys.stdout.buffer)
