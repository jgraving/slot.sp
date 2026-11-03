// Exercise the live 3D model, native autoplay and OS-aligned manual controls.
const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const {execFileSync}=require('node:child_process');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'../public');
const out=path.resolve(process.argv[2]);fs.mkdirSync(out,{recursive:true});
const origin='https://jakegraving.com/slot.sp/';
const mime={'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.png':'image/png','.webp':'image/webp','.mp4':'video/mp4','.woff2':'font/woff2','.json':'application/json'};
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
  const boxes=page=>page.evaluate(()=>Object.fromEntries(['.intro-wordmark','.intro h1','.sp-stage','.preview-controls','.features'].map(selector=>{const r=document.querySelector(selector).getBoundingClientRect();return [selector,{x:r.x,y:r.y+scrollY,width:r.width,height:r.height}];})));
  function stable(before,after,label){for(const selector of Object.keys(before))for(const property of ['x','y','width','height'])assert(Math.abs(before[selector][property]-after[selector][property])<1,`${label}: ${selector} ${property} shifted`);}
  async function fitsStage(page,label) {
    const sizes=await page.locator('.sp-stage').evaluate(stage=>{
      const box=stage.getBoundingClientRect(),image=stage.querySelector('.sp-render-fallback').getBoundingClientRect();
      return {width:box.width,height:box.height,imageWidth:image.width,imageHeight:image.height,decoderDisplay:getComputedStyle(stage.querySelector('.hero-source')).display};
    });
    assert.equal(sizes.decoderDisplay,'none',`${label}: decoder must never appear behind the handheld`);
    assert(Math.abs(sizes.height-sizes.width*1.25)<1,`${label}: preserve the complete 4:5 render`);
    assert(Math.abs(sizes.width-sizes.imageWidth)<1&&Math.abs(sizes.height-sizes.imageHeight)<1,`${label}: render must fit without cropping`);
  }
  const waitScreen=(page,state,cart)=>page.waitForFunction(([state,cart])=>{
    const el=document.getElementById('hero-screen');return el.dataset.state===state&&(cart===undefined||el.dataset.cart===String(cart));
  },[state,cart]);
  const keyButton=(page,key)=>page.locator(`[data-hardware=${key}]`);
  const holdMenu=async page=>{await keyButton(page,'menu').click({delay:1100});};
  async function silhouette(page,label) {
    const file=path.join(out,`${label}.png`);await page.locator('.sp-stage').screenshot({path:file});
    execFileSync('python3',['-c',`from PIL import Image,ImageChops
import sys
im=Image.open(sys.argv[1]).convert('RGB');bg=(14,20,35);w,h=im.size
for p in [(1,1),(w-2,1),(1,h-2),(w-2,h-2)]:
 assert max(abs(a-b) for a,b in zip(im.getpixel(p),bg))<=2, ('Visible canvas box',p,im.getpixel(p))
r,g,b=ImageChops.difference(im,Image.new('RGB',im.size,bg)).split()
box=ImageChops.lighter(ImageChops.lighter(r,g),b).point(lambda v:255 if v>35 else 0).getbbox()
assert box and box[0]>1 and box[1]>1 and box[2]<w-1 and box[3]<h-1, ('Clipped handheld',box,im.size)
`,file],{stdio:'inherit'});
  }
  async function rotate(page,label) {
    const stage=page.locator('.sp-stage'),box=await stage.boundingBox();
    const initial=Number(await stage.getAttribute('data-view-yaw'));
    await page.mouse.move(box.x+30,box.y+box.height*.6);await page.mouse.down();
    await page.mouse.move(box.x+170,box.y+box.height*.6+25,{steps:12});await page.mouse.up();
    await page.waitForFunction(y=>Math.abs(Number(document.querySelector('.sp-stage').dataset.viewYaw)-y)>.7,initial);
    await stage.focus();await page.keyboard.press('Home');
    await page.waitForFunction(()=>Math.abs(Number(document.querySelector('.sp-stage').dataset.viewYaw)+.38)<.002);
    // Rotate to the rear and tip the rear edge toward the camera. Every
    // shoulder target follows its own face normal, not the front D-pad plane.
    for(let i=0;i<23;i++)await page.keyboard.press('ArrowRight');
    for(let i=0;i<8;i++)await page.keyboard.press('ArrowDown');
    await page.waitForFunction(()=>Number(document.querySelector('.sp-stage').dataset.viewPitch)>.6);
    await silhouette(page,`${label}-shoulders`);
    for(const key of ['l1','l2','r1','r2']){
      assert(await keyButton(page,key).isVisible(),`${label}: ${key} is reachable from the rear`);
      await keyButton(page,key).click();
    }
    assert(!(await keyButton(page,'a').isVisible()),'Face buttons cannot be clicked through the back');
    await stage.focus();await page.keyboard.press('Home');
    await page.waitForFunction(()=>Math.abs(Number(document.querySelector('.sp-stage').dataset.viewYaw)+.38)<.002);
    await silhouette(page,`${label}-front`);
  }
  async function manualChecks(page) {
    // Start from the seated cartridge reached by autoplay. The first button
    // claims control even if its OS action is outside this menu-free preview.
    await keyButton(page,'b').click();
    assert.equal(await page.locator('#hero-screen').getAttribute('data-demo'),'false');
    await page.waitForTimeout(1600);await waitScreen(page,'seated',1);
    await keyButton(page,'left').click();await keyButton(page,'b').click();
    await keyButton(page,'menu').click();await page.waitForTimeout(250);
    await waitScreen(page,'seated',1);
    assert.equal(await page.locator('#hero-screen').getAttribute('data-boots'),'1','MENU no longer reboots');
    await holdMenu(page);await waitScreen(page,'shelf',1);
    await keyButton(page,'left').click();await waitScreen(page,'shelf',0);
    for(let cart=0;cart<5;cart++) {
      await keyButton(page,'a').click();await waitScreen(page,'seated',cart);
      assert(await page.locator('#hero-preview').evaluate(v=>v.paused));
      await page.locator('.sp-stage').screenshot({path:path.join(out,`cart-${cart}-seated.png`)});
      await holdMenu(page);await waitScreen(page,'shelf',cart);
      await keyButton(page,'right').click();await waitScreen(page,'shelf',(cart+1)%5);
    }
    await keyButton(page,'right').click({clickCount:3,delay:30});await waitScreen(page,'shelf',3);
    await keyButton(page,'left').click({clickCount:2,delay:30});await waitScreen(page,'shelf',1);
    await keyButton(page,'right').click();await keyButton(page,'left').click();
    await keyButton(page,'a').click();await waitScreen(page,'seated',1);
    await holdMenu(page);await waitScreen(page,'shelf',1);
    await keyButton(page,'down').click();await waitScreen(page,'shelf',2);
    await keyButton(page,'up').click();await waitScreen(page,'shelf',1);
    await keyButton(page,'right').focus();await page.keyboard.press('ArrowRight');await waitScreen(page,'shelf',2);
    await page.keyboard.press('a');await waitScreen(page,'seated',2);
    await page.keyboard.press('b');await waitScreen(page,'seated',2);
    await page.keyboard.down('m');await page.waitForTimeout(1100);await page.keyboard.up('m');await waitScreen(page,'shelf',2);
    const before=await boxes(page);
    await keyButton(page,'right').click();await page.locator('.model-fold').click();
    await page.locator('[data-pose=closed]').waitFor();
    const stopped=await page.locator('#hero-preview').evaluate(v=>v.currentTime);
    await page.waitForTimeout(300);assert.equal(await page.locator('#hero-preview').evaluate(v=>v.currentTime),stopped);
    await page.locator('.model-fold').click();await page.locator('[data-pose=open]').waitFor();await waitScreen(page,'shelf',3);
    stable(before,await boxes(page),'Manual controls and lid');
    assert.equal(await page.locator('#hero-screen').getAttribute('data-boots'),'1');
    await page.waitForTimeout(1600);await waitScreen(page,'shelf',3);
    assert.equal(await page.locator('#hero-screen').getAttribute('data-demo'),'false','Manual control never silently restarts autoplay');
    report.push({nativeActions:true,osMappings:true,menuHold:true,autoplayTakeover:true,wrap:true,rapidInput:true,keyboard:true});
  }
  try {
    for(const width of [1366,390,320]) {
      const page=await pageFor(width,{reducedMotion:'no-preference'}),errors=[],requests=[];
      page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>requests.push(r.url()));
      page.on('console',m=>{if(m.type()==='warning'||m.type()==='error')console.log('Browser:',m.type(),m.text());});
      let release;const held=new Promise(resolve=>release=resolve);
      await page.route('**/sp-model.*.js',async route=>{await held;await route.fallback();});
      await page.goto(origin,{waitUntil:'domcontentloaded'});await page.locator('.sp-stage').scrollIntoViewIfNeeded();
      await page.evaluate(()=>document.fonts.ready);
      const before=await boxes(page);await fitsStage(page,`${width}: first paint`);
      assert(await page.locator('.model-poster').isVisible(),'Reserve a complete model while WebGL loads');
      // Observe boot while it happens. Software WebGL can delay the next
      // automation round trip beyond a fixed video timestamp.
      await page.evaluate(()=>{
        const canvas=document.getElementById('hero-screen'),video=document.getElementById('hero-preview');
        let first=null;
        window.__nativeBootSamples=[];
        const frame=()=>{
          if(canvas.dataset.state!=='boot')return;
          if(window.__nativeBootSamples.length<20)window.__nativeBootSamples.push({time:video.currentTime,paused:video.paused,ready:video.readyState,at:performance.now()});
          if(video.paused)return;
          const pixels=canvas.toDataURL('image/jpeg',.1);
          if(first===null)first=pixels;
          else if(first!==pixels){window.__nativeBootChanged=true;canvas.removeEventListener('hero:frame',frame);}
        };
        canvas.addEventListener('hero:frame',frame);
      });
      release();
      await page.locator('.sp-stage.model-ready[data-pose=open]').waitFor({timeout:60000});
      if(width!==1366)await keyButton(page,'b').click();
      try {await page.waitForFunction(()=>window.__nativeBootChanged===true);}
      catch(error){
        console.log('Boot playback diagnostics',await page.evaluate(()=>({frames:window.__nativeBootSamples,screen:{...document.getElementById('hero-screen').dataset},video:{time:document.getElementById('hero-preview').currentTime,paused:document.getElementById('hero-preview').paused}})));
        await page.screenshot({path:path.join(out,`${width}-boot-failure.png`)});throw error;
      }
      assert.equal(await page.locator('#hero-screen').getAttribute('data-failed'),'false');
      stable(before,await boxes(page),`${width}: renderer handoff`);
      assert(requests.some(url=>url.includes('/vendor/three')),'The live 3D model must load');
      assert.equal(await page.locator('.model-opening').getAttribute('src'),null,'Do not download a second model animation with WebGL');
      if(width===1366){await waitScreen(page,'seated',1);await manualChecks(page);}
      else await waitScreen(page,'shelf',0);
      await rotate(page,String(width));
      await page.locator('.model-fold').click();await page.locator('[data-pose=closed]').waitFor();
      assert(await page.locator('#hero-preview').evaluate(v=>v.paused));
      await silhouette(page,`${width}-closed`);
      await page.locator('.model-fold').click();await page.locator('[data-pose=open]').waitFor();
      stable(before,await boxes(page),`${width}: rotating and lid`);
      if(width===1366)await page.screenshot({path:path.join(out,'desktop-home.png')});
      if(width===390)await page.screenshot({path:path.join(out,'phone-home.png')});
      assert.deepEqual(errors,[],`${width}: runtime errors`);
      report.push({width,rotation:true,opaqueModel:true,layoutStable:true,bootOnce:true,errors});await page.close();
    }
    for(const width of [1366,820,390,320]) {
      const page=await pageFor(width),errors=[],requests=[];page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>requests.push(r.url()));
      await page.goto(origin,{waitUntil:'networkidle'});await page.locator('.sp-stage').scrollIntoViewIfNeeded();
      await page.locator('.sp-stage.model-ready[data-pose=open]').waitFor();await page.locator('#hero-screen[data-painted=true]').waitFor({state:'attached'});
      assert.equal(await page.locator('#hero-screen').getAttribute('data-demo'),'false');
      assert(await page.locator('#hero-preview').evaluate(v=>v.paused));
      assert(!requests.some(url=>url.endsWith('.mp4')),'Reduced motion downloads no animations');
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
      assert((await page.locator('.intro-wordmark').boundingBox()).width>=190);
      for(const key of ['a','b','x','y','menu','select','start','left','right','up','down']) {
        const hit=await keyButton(page,key).evaluate(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)===el;});
        assert(hit,`${width}: ${key} target must track its physical button`);
        const offset=await keyButton(page,key).evaluate(el=>{
          const r=el.getBoundingClientRect(),stage=el.closest('.sp-stage').getBoundingClientRect();
          return Math.hypot(r.x+r.width/2-stage.x-parseFloat(el.style.left)*stage.width/100,r.y+r.height/2-stage.y-parseFloat(el.style.top)*stage.height/100);
        });
        assert(offset<1,`${width}: ${key} hit target is centered on its projected mesh`);
      }
      await keyButton(page,'right').click();await waitScreen(page,'shelf',1);
      await keyButton(page,'a').click();await waitScreen(page,'seated',1);
      await holdMenu(page);await waitScreen(page,'shelf',1);
      for(const name of ['Graphite','Pearl Blue','Pearl Pink','Platinum']) {
        await page.getByRole('button',{name,exact:true}).click();
        await page.waitForFunction(f=>document.getElementById('hero-screen').dataset.finish===f,name.toLowerCase().replaceAll(' ','_'));
        assert.equal(await page.locator('.sp-stage').getAttribute('data-finish'),name.toLowerCase());
        if(width===1366)await silhouette(page,name.toLowerCase().replaceAll(' ','-'));
      }
      const before=await boxes(page);await page.locator('.model-fold').press('Space');await page.locator('[data-pose=closed]').waitFor();
      await page.locator('.model-fold').press('Enter');await page.locator('[data-pose=open]').waitFor();stable(before,await boxes(page),'Reduced-motion lid');
      if(width===1366){
        await page.getByRole('button',{name:'Pearl Blue',exact:true}).click();await page.reload({waitUntil:'networkidle'});
        await page.locator('.sp-stage[data-finish="pearl blue"]').waitFor();
        assert.equal(await page.locator('.finish-name').textContent(),'Pearl Blue');
        for(const key of ['ArrowUp','ArrowUp','ArrowDown','ArrowDown','ArrowLeft','ArrowRight','ArrowLeft','ArrowRight','b','a'])await page.keyboard.press(key);
        await page.locator('.sp-stage[data-finish=gold]').waitFor();
      }
      assert.deepEqual(errors,[]);report.push({width,reducedMotion:true,hitTargets:true,finishes:true});await page.close();
    }
    const saved=await pageFor(390,{reducedMotion:'no-preference'});
    await saved.addInitScript(()=>Object.defineProperty(navigator,'connection',{value:{saveData:true}}));
    await saved.goto(origin,{waitUntil:'networkidle'});await saved.locator('.sp-stage.model-ready[data-pose=open]').waitFor();
    assert.equal(await saved.locator('#hero-preview').getAttribute('src'),null);assert.equal(await saved.locator('#hero-screen').getAttribute('data-demo'),'false');
    await saved.close();fs.writeFileSync(path.join(out,'checks.json'),JSON.stringify(report,null,2));
    console.log('PASS: rotating 3D, shoulder hit targets, boot-first autoplay, manual takeover, OS controls, stable layout, finishes and reduced motion');
  }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
