"""Inspect actual serialized WASM output; preview uses Pillow, not Deck's GPU."""
import struct as s
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
class Reader:
 def __init__(self,b):self.b=b;self.p=0
 def read(self,f):v=s.unpack_from('<'+f,self.b,self.p);self.p+=s.calcsize('<'+f);return v[0] if len(v)==1 else v
 def string(self):n=self.read('H');v=self.b[self.p:self.p+n].decode();self.p+=n;return v
 def node(self):
  tag=self.read('B');raw=self.b[self.p:self.p+78];self.p+=78
  pad,margin,gap,bg,w,h=s.unpack_from('<fffIff',raw);props=(pad,gap,bg,w,h)
  if tag in (0,1):return tag,props,[self.node() for _ in range(self.read('H'))]
  assert tag==6,tag
  key=self.string();draws=[]
  for _ in range(self.read('H')):
   t=self.read('B');box=self.read('ffff')
   if t==0x40:assert self.read('B')==0;draws.append(('rect',box,self.read('I')))
   elif t==0x4b:
    self.read('BHH');style=self.read('IIIIIfI');text=self.string();draws.append(('text',box,style[1],style[0],text))
   elif t==0x44:
    self.read('IHB');n=self.read('H')
    for _ in range(n):self.string();self.read('I')
   else:raise AssertionError(t)
  return tag,props,(key,draws)
def color(n):return ((n>>24)&255,(n>>16)&255,(n>>8)&255)
def preview(width,height,menu=False,donation=None,panel=None):
 kind=panel if panel else f"donation{donation}" if donation is not None else "menu" if menu else "cz"
 data=Path(f'/tmp/radio-{kind}-{width}-{height}.tree').read_bytes();r=Reader(data);tree=r.node();assert r.p==len(data)
 im=Image.new('RGB',(width,height));draw=ImageDraw.Draw(im);regions=[]
 def visit(node,x,y):
  tag,(pad,gap,bg,w,h),content=node
  if tag in (0,1):
   if bg&255:draw.rectangle((x,y,x+w,y+h),fill=color(bg))
   cx=x+pad;cy=y+pad;mw=mh=0
   for child in content:
    cw,ch=visit(child,cx,cy);mw=max(mw,cw);mh=max(mh,ch)
    if tag==0:cy+=ch+gap
    else:cx+=cw+gap
   return (w or (cx-x-gap+pad if tag==1 else mw+2*pad),h or (cy-y-gap+pad if tag==0 else mh+2*pad))
  key,cmds=content
  if key:
   assert x>=0 and y>=0 and x+w<=width+.1 and y+h<=height+.1,(key,x,y,w,h)
   assert w>=48 and h>=48,(key,w,h)
   regions.append((x,y,w,h))
  for cmd in cmds:
   kind,(dx,dy,dw,dh),co,*rest=cmd;xx=x+dx;yy=y+dy
   if kind=='rect':draw.rectangle((xx,yy,xx+dw-1,yy+dh-1),fill=color(co))
   else:
    size,text=rest;size=int(size)
    fontpath='bmc-main/assets/fonts/NotoSans-Regular.ttf'
    font=ImageFont.truetype(fontpath,size)
    while (font.getlength(text)>dw or size>dh) and size>7:size-=1;font=ImageFont.truetype(fontpath,size)
    draw.text((xx+dw/2,yy+dh/2),text,font=font,anchor='mm',fill=color(co))
  return w,h
 visit(tree,0,0)
 assert len(regions)==(6 if panel=="list" else 8 if panel=="settings" else 1 if donation is not None else 8 if menu else (21 if width>=960 else 17 if width>=600 else 12))
 for i,(x,y,w,h) in enumerate(regions):
  for X,Y,W,H in regions[i+1:]:assert x+w<=X+.1 or X+W<=x+.1 or y+h<=Y+.1 or Y+H<=y+.1
 out=Path(f'/tmp/radio-preview-{kind}-{width}-{height}.png');im.save(out);print('PASS bounds,',len(regions),'non-overlapping touch areas >=48 px:',out)
for wh in [(317,238),(638,238),(638,480),(1280,480)]:preview(*wh)

for wh in [(317,238),(638,238),(638,480),(1280,480)]:preview(*wh,menu=True)

for i in range(4):
 for wh in [(317,238),(638,238),(638,480),(1280,480)]:preview(*wh,donation=i)

for panel in ["list","settings"]:
 for wh in [(317,238),(638,238),(638,480),(1280,480)]:preview(*wh,panel=panel)
