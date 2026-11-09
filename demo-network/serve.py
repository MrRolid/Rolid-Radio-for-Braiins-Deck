#!/usr/bin/env python3
"""Read-only, range-aware MP3 demo server. Run on your PC/NAS, not on Deck."""
import argparse,http.server,os,re,socketserver
from pathlib import Path
class Handler(http.server.SimpleHTTPRequestHandler):
 def send_head(self):
  p=Path(self.translate_path(self.path))
  if not p.is_file():return super().send_head()
  size=p.stat().st_size;start,end=0,size-1;partial=False
  if self.headers.get('Range'):
   m=re.fullmatch(r'bytes=(\d+)-(\d*)',self.headers['Range'])
   if not m:self.send_error(416);return None
   start=int(m[1]);end=min(int(m[2]) if m[2] else end,end);partial=True
   if start>end:self.send_error(416);return None
  f=p.open('rb');f.seek(start);self.remaining=end-start+1
  self.send_response(206 if partial else 200);self.send_header('Content-Type',self.guess_type(str(p)));self.send_header('Content-Length',str(self.remaining));self.send_header('Accept-Ranges','bytes')
  if partial:self.send_header('Content-Range',f'bytes {start}-{end}/{size}')
  self.end_headers();return f
 def copyfile(self,source,output):
  if not hasattr(self,'remaining'):return super().copyfile(source,output)
  try:
   while self.remaining:
    b=source.read(min(65536,self.remaining))
    if not b:break
    output.write(b);self.remaining-=len(b)
  except (BrokenPipeError,ConnectionResetError):pass
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',default=str(Path(__file__).parent));p.add_argument('--port',type=int,default=18767);a=p.parse_args();os.chdir(a.directory)
 print(f'Add http://<PC-or-NAS-IP>:{a.port}/demo.mp3 in the Radio network library.',flush=True)
 http.server.ThreadingHTTPServer(('0.0.0.0',a.port),Handler).serve_forever()
