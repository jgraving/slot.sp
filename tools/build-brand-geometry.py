"""Export actual variable-font contours for the physically rendered brand reveal."""
import io,json,math,sys,urllib.request
import xml.etree.ElementTree as ET
from fontTools.svgLib.path import parse_path
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.basePen import BasePen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen

class Contours(BasePen):
    def __init__(self,glyphset):
        super().__init__(glyphset);self.contours=[];self.points=[]
    def _moveTo(self,p): self.points=[p]
    def _lineTo(self,p): self.points.append(p)
    def _qCurveToOne(self,p1,p2):
        p0=self._getCurrentPoint();length=sum(math.dist(a,b) for a,b in [(p0,p1),(p1,p2)])
        for i in range(1,max(5,math.ceil(length/22))+1):
            t=i/max(5,math.ceil(length/22));self.points.append(tuple((1-t)**2*a+2*(1-t)*t*b+t*t*c for a,b,c in zip(p0,p1,p2)))
    def _curveToOne(self,p1,p2,p3):
        p0=self._getCurrentPoint();length=sum(math.dist(a,b) for a,b in [(p0,p1),(p1,p2),(p2,p3)])
        n=max(8,math.ceil(length/22))
        for i in range(1,n+1):
            t=i/n;self.points.append(tuple((1-t)**3*a+3*(1-t)**2*t*b+3*(1-t)*t*t*c+t**3*d for a,b,c,d in zip(p0,p1,p2,p3)))
    def _closePath(self):
        if len(self.points)>2:self.contours.append(self.points)
        self.points=[]
    def _endPath(self):self._closePath()

def generate(path,font_path=None):
    data=Path(font_path).read_bytes() if font_path else urllib.request.urlopen('https://jakegraving.com/assets/fonts/google-sans-flex-v5-web.woff2',timeout=60).read()
    font=TTFont(io.BytesIO(data));cmap=font.getBestCmap();em=font['head'].unitsPerEm
    final={'opsz':18,'wdth':100,'wght':600,'ROND':0,'slnt':0}
    frames=[]
    for frame in range(48):
        t=frame/47
        q=min(t/.82,1);ease=q*q*(3-2*q)
        # One directed material transformation, with a restrained weight overshoot.
        loc={'opsz':72-54*ease,'wdth':108-8*ease,
             'wght':350+250*ease+65*math.sin(math.pi*q)**4,
             'ROND':60*(1-ease),'slnt':-6*(1-ease)}
        if t>=.82:loc=final.copy()
        gs=font.getGlyphSet(location=loc);glyphs=[];advance=0.
        for i,ch in enumerate('slot.sp'):
            glyph=gs[cmap[ord(ch)]];pen=Contours(gs);glyph.draw(pen)
            glyphs.append({'name':ch,'contours':[[[(x+advance)/em,y/em] for x,y in c] for c in pen.contours]})
            advance+=glyph.width
        # Centre each optical width. Use a fixed final baseline centre, avoiding bobbing.
        for glyph in glyphs:
            for contour in glyph['contours']:
                for point in contour:
                    point[0]=round((point[0]-advance/em/2)/.933,6)
                    point[1]=round((point[1]-.25)/.933,6)
        frames.append({'t':t,'axes':loc,'glyphs':glyphs})
    Path(path).write_text(json.dumps({'frames':frames},separators=(',',':')))
    # Export the exact same final contours as reusable, font-independent SVG.
    root=Path(__file__).resolve().parents[1]
    def lockup_svg(width,height,scale,background=''):
        paths=[];finalset=font.getGlyphSet(location=final)
        total=sum(finalset[cmap[ord(c)]].width for c in 'slot.sp');advance_x=0
        for ch in 'slot.sp':
            pen=SVGPathPen(finalset)
            matrix=(scale/em/.933,0,0,-scale/em/.933,
                    width/2+(advance_x/em-total/em/2)/.933*scale,
                    height/2+.25/.933*scale)
            glyph=finalset[cmap[ord(ch)]];glyph.draw(TransformPen(pen,matrix));advance_x+=glyph.width
            paths.append('<path d="'+pen.getCommands()+'"/>')
        return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" role="img" aria-label="slot.sp">'+background+'<g fill="#f6f7f2">'+''.join(paths)+'</g></svg>'
    (root/'assets/slot-sp-logo.svg').write_text(lockup_svg(800,200,145))
    bg='<defs><radialGradient id="g"><stop stop-color="#253a4a"/><stop offset="1" stop-color="#0b0e11"/></radialGradient></defs><path fill="url(#g)" d="M0 0H720V480H0Z"/>'
    (root/'previews/hero-static.svg').write_text(lockup_svg(720,480,118,bg))
    print('geometry',path,Path(path).stat().st_size)

if __name__=='__main__': generate(sys.argv[1],sys.argv[2] if len(sys.argv)>2 else None)
