// Exercise the optional WebGL hero against built files, without external assets.
const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'../public');
const out=path.resolve(process.argv[2]);fs.mkdirSync(out,{recursive:true});
const origin='https://jakegraving.com/slot.sp/';
const mime={'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.png':'image/png','.mp4':'video/mp4','.woff2':'font/woff2','.json':'application/json'};
(async()=>{
  const browser=await chromium.launch({executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
  const report=[];
  async function pageFor(width,options={}) {
    const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:1.5,reducedMotion:'reduce',...options});
    await page.route(origin+'**',async route=>{
      let file=path.resolve(root,decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
      if(!file.startsWith(root+path.sep)&&file!==root)return route.fulfill({status:403});
      if(fs.existsSync(file)&&fs.statSync(file).isDirectory())file=path.join(file,'index.html');
      if(!fs.existsSync(file))return route.fulfill({status:404});
      const contentType=mime[path.extname(file)]||'application/octet-stream';
      // Media seeking needs the same byte-range semantics as GitHub Pages.
      if(path.extname(file)==='.mp4') {
        const bytes=fs.readFileSync(file),range=route.request().headers().range;
        const match=range?.match(/^bytes=(\d+)-(\d*)$/);
        if(match) {
          const start=Number(match[1]),end=Math.min(match[2]?Number(match[2]):bytes.length-1,bytes.length-1);
          return route.fulfill({status:206,body:bytes.subarray(start,end+1),contentType,headers:{'accept-ranges':'bytes','content-range':`bytes ${start}-${end}/${bytes.length}`,'content-length':String(end-start+1)}});
        }
        return route.fulfill({body:bytes,contentType,headers:{'accept-ranges':'bytes','content-length':String(bytes.length)}});
      }
      await route.fulfill({path:file,contentType});
    });return page;
  }
  try {
    for(const width of [1366,390,320]) {
      const page=await pageFor(width);const errors=[];
      page.on('pageerror',e=>errors.push(e.message));
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
      await page.goto(origin,{waitUntil:'networkidle'});
      const stage=page.locator('.sp-stage');await stage.scrollIntoViewIfNeeded();
      await page.locator('[data-model-ready="true"][data-screen-ready="true"]').waitFor({timeout:45000});
      await page.evaluate(()=>document.fonts.ready);
      assert(await page.evaluate(()=>document.getElementById('hero-preview').paused),'Reduced motion must stay paused');
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'Horizontal overflow');
      await stage.screenshot({path:path.join(out,`${width}-model.png`)});
      if(width===1366)await page.screenshot({path:path.join(out,'desktop-home.png')});
      if(width===390)await page.screenshot({path:path.join(out,'phone-home.png')});
      for(const name of ['A: insert cartridge preview','B: eject cartridge preview','MENU: replay startup','D-pad left: previous cartridge preview','D-pad right: next cartridge preview']) {
        const control=page.getByRole('button',{name,exact:true});
        const hit=await control.evaluate(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)===el;});
        assert(hit,`${width}: ${name} obscured`);
      }
      // Keep media diagnostics in the artifact when a browser rejects playback.
      await page.evaluate(()=>{
        const v=document.getElementById('hero-preview');window.mediaTrace=[];
        for(const type of ['loadstart','loadedmetadata','loadeddata','play','playing','pause','seeking','seeked','timeupdate','error'])v.addEventListener(type,()=>window.mediaTrace.push({type,time:v.currentTime,paused:v.paused,hidden:document.hidden}));
        const play=v.play.bind(v);v.play=()=>play().catch(error=>{window.mediaTrace.push({type:'play-rejected',message:error.message});throw error;});
        document.querySelector('[data-hardware=insert]').addEventListener('click',()=>window.mediaTrace.push({type:'A-click'}));
      });
      // Screen actions remain the native clip, and stop at their recorded endpoint.
      await page.getByRole('button',{name:'A: insert cartridge preview',exact:true}).click();
      try {
        await page.waitForFunction(()=>{const v=document.getElementById('hero-preview');return v.paused && Math.abs(v.currentTime-1.1)<.08;},{},{timeout:15000});
      } catch(error) {
        fs.writeFileSync(path.join(out,'video-failure.json'),JSON.stringify(await page.evaluate(()=>{const v=document.getElementById('hero-preview');return {paused:v.paused,time:v.currentTime,ready:v.readyState,error:v.error?.message,src:v.currentSrc,duration:v.duration,muted:v.muted,trace:window.mediaTrace,lid:document.querySelector('.sp-stage').dataset.lid,visibility:getComputedStyle(v).visibility};}),null,2));
        fs.writeFileSync(path.join(out,'browser-errors.json'),JSON.stringify(errors,null,2));
        await stage.screenshot({path:path.join(out,'video-failure.png')});throw error;
      }
      await page.getByRole('button',{name:'Graphite',exact:true}).click();
      await page.waitForFunction(()=>document.getElementById('hero-preview').poster.includes('graphite'));
      if(width===1366)await stage.screenshot({path:path.join(out,'graphite-model.png')});
      await page.getByRole('button',{name:'Platinum',exact:true}).click();
      await page.getByRole('button',{name:'Close lid',exact:true}).click();
      await page.waitForFunction(()=>document.querySelector('.sp-stage').dataset.lid==='closed');
      assert(!(await page.getByRole('button',{name:'A: insert cartridge preview',exact:true}).isVisible()));
      if(width===1366)await stage.screenshot({path:path.join(out,'closed-model.png')});
      await page.getByRole('button',{name:'Open lid',exact:true}).click();
      await stage.focus();await page.keyboard.press('ArrowRight');await page.keyboard.press('ArrowRight');
      if(width===1366)await stage.screenshot({path:path.join(out,'turned-model.png')});
      await page.keyboard.press('Home');
      assert.deepEqual(errors,[],`Browser errors at ${width}`);
      report.push({width,webgl:true,overflow:false,controls:true,reducedMotion:true,errors});await page.close();
    }
    const fallback=await pageFor(390);
    await fallback.addInitScript(()=>{const get=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(type,...args){return type.startsWith('webgl')?null:get.call(this,type,...args);};});
    await fallback.goto(origin,{waitUntil:'networkidle'});
    await fallback.locator('.sp-stage').scrollIntoViewIfNeeded();
    assert(await fallback.locator('.device').isVisible());assert(await fallback.getByRole('button',{name:'A: insert cartridge preview',exact:true}).isVisible());
    assert(!(await fallback.locator('.model-fold').isVisible()));
    await fallback.screenshot({path:path.join(out,'no-webgl.png')});await fallback.close();
    fs.writeFileSync(path.join(out,'checks.json'),JSON.stringify(report,null,2));
    console.log('PASS: 3D desktop and mobile renders, native clip actions, color changes, fold, keyboard rotation, reduced motion and WebGL fallback');
  }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
