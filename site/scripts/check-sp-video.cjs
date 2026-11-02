// Composited video regression: all finishes, both lid directions, Chromium/WebKit.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {execFileSync}=require('node:child_process');
const {chromium,webkit}=require('playwright');
const root=path.resolve(__dirname,'../public'),out=path.resolve(process.argv[2]);
fs.mkdirSync(out,{recursive:true});
const origin='https://jakegraving.com/slot.sp/';
const mime={'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.png':'image/png','.webp':'image/webp','.mp4':'video/mp4','.woff2':'font/woff2','.json':'application/json'};
(async()=>{
 const results=[];
 for(const [name,engine] of [['chromium',chromium],['webkit',webkit]]){
  const browser=await engine.launch(name==='chromium'?{executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']}:{});
  try{
   for(const finish of ['pearl blue','platinum','graphite','pearl pink']){
    const page=await browser.newPage({viewport:{width:1366,height:1000},reducedMotion:'no-preference'});
    const errors=[];page.on('pageerror',e=>errors.push(e.message));
    await page.route(origin+'**',async route=>{
     let file=path.resolve(root,decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
     if(!file.startsWith(root+path.sep)&&file!==root)return route.fulfill({status:403});
     if(fs.existsSync(file)&&fs.statSync(file).isDirectory())file=path.join(file,'index.html');
     if(!fs.existsSync(file))return route.fulfill({status:404});
     const contentType=mime[path.extname(file)]||'application/octet-stream';
     if(path.extname(file)==='.mp4'){
      const bytes=fs.readFileSync(file),match=route.request().headers().range?.match(/^bytes=(\d+)-(\d*)$/);
      if(match){
       const start=Number(match[1]),end=Math.min(match[2]?Number(match[2]):bytes.length-1,bytes.length-1);
       return route.fulfill({status:206,body:bytes.subarray(start,end+1),contentType,headers:{'accept-ranges':'bytes','content-range':`bytes ${start}-${end}/${bytes.length}`,'content-length':String(end-start+1)}});
      }
      return route.fulfill({body:bytes,contentType,headers:{'accept-ranges':'bytes','content-length':String(bytes.length)}});
     }
     return route.fulfill({path:file,contentType});
    });
    await page.addInitScript(finish=>{
     localStorage.setItem('slot.sp:hardware-finish',finish);
     // Hold the first presented frame for deterministic screenshot sampling.
     document.addEventListener('loadeddata',event=>{
      const video=event.target;
      if(video.matches?.('.model-opening'))video.requestVideoFrameCallback(()=>video.pause());
     },true);
    },finish);
    await page.goto(origin,{waitUntil:'domcontentloaded'});
    await page.locator('.sp-stage[data-motion-ready=true][data-pose=opening]').waitFor();
    assert.equal(await page.locator('.finish-name').textContent(),finish.replace(/\b\w/g,c=>c.toUpperCase()));
    for(const direction of ['opening','closing']){
     if(direction==='closing'){
      await page.getByRole('button',{name:'Close lid',exact:true}).click();
      await page.locator('.sp-stage[data-motion-ready=true][data-pose=closing]').waitFor();
     }
     const video=page.locator('.model-opening');
     const style=await video.evaluate(v=>({filter:getComputedStyle(v).filter,blend:getComputedStyle(v).mixBlendMode,still:getComputedStyle(document.querySelector('.model-poster')).visibility}));
     assert.equal(style.filter,'none','Never apply an SVG filter to the decoded video');
     assert.equal(style.blend,'lighten');assert.equal(style.still,'hidden','The old lid cannot show through the playing frame');
     for(const time of [.1,.45,.8]){
      await video.evaluate(async(v,time)=>{
       v.pause();await new Promise(resolve=>{v.addEventListener('seeked',resolve,{once:true});v.currentTime=time;});
      },time);
      const file=path.join(out,`${name}-${finish.replaceAll(' ','-')}-${direction}-${time}.png`);
      await page.locator('.sp-stage').screenshot({path:file});
      execFileSync('python3',['-c',`from PIL import Image
import sys
im=Image.open(sys.argv[1]).convert('RGB');w,h=im.size;expected=(14,20,35)
points=[(x,y) for x in [1,w-2] for y in [1,h//4,h//2,3*h//4,h-2]]
points += [(x,y) for y in [1,h-2] for x in [w//4,w//2,3*w//4]]
for point in points:
 actual=im.getpixel(point)
 assert max(abs(a-b) for a,b in zip(actual,expected))<=2, ('Visible video rectangle',point,actual,expected,sys.argv[1])
`,file]);
     }
     await video.evaluate(v=>{v.currentTime=v.duration-.03;v.play();});
     await page.locator(`[data-pose=${direction==='opening'?'open':'closed'}]`).waitFor();
    }
    assert.deepEqual(errors,[],`${name} ${finish}: runtime errors`);
    results.push({engine:name,finish,opening:true,closing:true,frames:6,matte:true});
    await page.close();
   }
  }finally{await browser.close();}
 }
 fs.writeFileSync(path.join(out,'video-checks.json'),JSON.stringify(results,null,2));
 console.log('PASS: 48 composited opening/closing frames, all four finishes including saved Pearl Blue, Chromium and WebKit');
})().catch(error=>{console.error(error);process.exitCode=1;});
