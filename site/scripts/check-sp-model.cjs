// Check the fixed-angle model, stable first load and native screen controls.
const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'../public');
const out=path.resolve(process.argv[2]);fs.mkdirSync(out,{recursive:true});
const origin='https://jakegraving.com/slot.sp/';
const mime={'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.png':'image/png','.mp4':'video/mp4','.woff2':'font/woff2','.json':'application/json'};
(async()=>{
  const browser=await chromium.launch({executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']});
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
  const boxes=page=>page.evaluate(()=>Object.fromEntries(['.intro-wordmark','.intro h1','.sp-stage','.preview-controls','.model-hint','.features'].map(selector=>{const r=document.querySelector(selector).getBoundingClientRect();return [selector,{x:r.x,y:r.y+scrollY,width:r.width,height:r.height}];})));
  function stable(before,after,label){for(const selector of Object.keys(before))for(const property of ['x','y','width','height'])assert(Math.abs(before[selector][property]-after[selector][property])<1,`${label}: ${selector} ${property} shifted`);}
  try {
    for(const width of [1366,390,320]) {
      const page=await pageFor(width,{reducedMotion:'no-preference'}),errors=[],requests=[];
      page.on('pageerror',e=>errors.push(e.message));
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
      page.on('request',r=>requests.push(r.url()));
      let release;
      const held=new Promise(resolve=>release=resolve);
      await page.route('**/*-opening.mp4',async route=>{await held;await route.fallback();});
      await page.goto(origin,{waitUntil:'domcontentloaded'});
      await page.locator('.sp-stage').scrollIntoViewIfNeeded();
      await page.evaluate(()=>document.fonts.ready);
      await page.locator('[data-pose=opening]').waitFor();
      assert(await page.evaluate(()=>document.getElementById('hero-preview').paused),'Hold screen playback during opening');
      const before=await boxes(page);
      if(width===1366)await page.screenshot({path:path.join(out,'first-load.png')});
      release();
      await page.locator('[data-pose=open]').waitFor();
      await page.waitForFunction(()=>document.querySelector('.model-poster img').complete);
      stable(before,await boxes(page),`${width}: first opening`);
      assert(await page.locator('.model-opening').evaluate(v=>v.currentTime>.9&&!v.error),'The rendered opening must play');
      assert.equal(await page.locator('canvas,.pose-hero,.device,.model-angle').count(),0,'No rotation renderer, old illustration or angle toolbar');
      assert(!requests.some(url=>/\/vendor\/three|\/sp-model\.[^/]*js/.test(url)),'Do not load a rotatable 3D runtime');
      const stage=page.locator('.sp-stage'),box=await stage.boundingBox();
      await page.mouse.move(box.x+12,box.y+80);await page.mouse.down();await page.mouse.move(box.x+box.width-12,box.y+120,{steps:8});await page.mouse.up();
      assert.equal(await stage.getAttribute('data-pose'),'open','Dragging must leave the fixed view alone');
      await page.getByRole('button',{name:'Close lid',exact:true}).click();
      await page.locator('[data-pose=closed]').waitFor();
      assert(await page.evaluate(()=>document.getElementById('hero-preview').paused),'Closed lid pauses the native screen');
      assert(!(await page.getByRole('button',{name:'A: insert cartridge preview',exact:true}).isVisible()));
      assert(await page.locator('.model-opening').evaluate(v=>v.currentTime>.9&&!v.error),'Closing must play its matching render');
      stable(before,await boxes(page),`${width}: lid closed`);
      if(width===1366)await stage.screenshot({path:path.join(out,'closed-model.png')});
      await page.getByRole('button',{name:'Open lid',exact:true}).click();await page.locator('[data-pose=open]').waitFor();
      stable(before,await boxes(page),`${width}: lid reopened`);
      await page.getByRole('button',{name:'Pause preview',exact:true}).click();
      if(width===1366)await page.screenshot({path:path.join(out,'desktop-home.png')});
      if(width===390)await page.screenshot({path:path.join(out,'phone-home.png')});
      assert.deepEqual(errors,[],`${width}: opening browser errors`);
      report.push({width,opening:true,closing:true,fixedView:true,layoutStable:true,errors});await page.close();
    }
    for(const width of [1366,820,390,320]) {
      const page=await pageFor(width),errors=[];
      page.on('pageerror',e=>errors.push(e.message));
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
      await page.goto(origin,{waitUntil:'networkidle'});
      const stage=page.locator('.sp-stage');await stage.scrollIntoViewIfNeeded();
      await page.locator('[data-ready=true][data-pose=open]').waitFor();
      await page.evaluate(()=>document.fonts.ready);
      assert(await page.evaluate(()=>document.getElementById('hero-preview').paused),'Reduced motion stays paused');
      assert.equal(await page.locator('.model-opening').getAttribute('src'),null,'Reduced motion never downloads the opening');
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'Horizontal overflow');
      const brand=page.locator('.intro-wordmark');assert(await brand.isVisible());
      assert((await brand.boundingBox()).width>=190,'Prominent homepage wordmark');
      for(const name of ['A: insert cartridge preview','B: eject cartridge preview','MENU: replay startup','D-pad left: previous cartridge preview','D-pad right: next cartridge preview']) {
        const hit=await page.getByRole('button',{name,exact:true}).evaluate(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)===el;});
        assert(hit,`${width}: ${name} is obscured`);
      }
      for(const [action,end]of width===1366?[['insert',1.1],['eject',3],['right',5.13],['left',7.38],['boot',9.25]]:[['insert',1.1]]) {
        await page.locator(`[data-hardware=${action}]`).click();
        await page.waitForFunction(end=>{const v=document.getElementById('hero-preview');return v.paused&&Math.abs(v.currentTime-end)<.08;},end,{timeout:15000});
      }
      for(const name of ['Graphite','Pearl Blue','Pearl Pink','Platinum']) {
        await page.getByRole('button',{name,exact:true}).click();
        await page.waitForFunction(finish=>document.querySelector('.model-poster img').currentSrc.includes(finish.replaceAll(' ','_')+'-open.png'),name.toLowerCase());
        assert.equal(await page.locator('.finish-name').textContent(),name);
        if(width===1366)await stage.screenshot({path:path.join(out,`${name.toLowerCase().replaceAll(' ','-')}-model.png`)});
      }
      const before=await boxes(page);
      await page.getByRole('button',{name:'Close lid',exact:true}).press('Space');
      await page.locator('[data-pose=closed]').waitFor();
      assert((await page.locator('.model-poster img').getAttribute('src')).endsWith('platinum-closed.png'));
      await page.getByRole('button',{name:'Open lid',exact:true}).press('Enter');await page.locator('[data-pose=open]').waitFor();
      stable(before,await boxes(page),`${width}: reduced-motion lid flip`);
      await page.locator('.preview-controls').scrollIntoViewIfNeeded();
      assert(await page.locator('.preview-toggle,.model-fold,[data-shell]').evaluateAll(nodes=>nodes.every(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('button')===el;})),`${width}: controls are covered`);
      if(width===1366) {
        await page.getByRole('button',{name:'Pearl Blue',exact:true}).click();await page.reload({waitUntil:'networkidle'});
        await stage.scrollIntoViewIfNeeded();await page.locator('[data-finish="pearl blue"]').waitFor();
        assert.equal(await page.locator('.finish-name').textContent(),'Pearl Blue','Remember the selected finish');
        for(const key of ['ArrowUp','ArrowUp','ArrowDown','ArrowDown','ArrowLeft','ArrowRight','ArrowLeft','ArrowRight','b','a'])await page.keyboard.press(key);
        await page.locator('[data-finish=gold]').waitFor();
        assert.equal(await page.locator('.finish-name').textContent(),'Gold','Keep the gold-shell secret');
      }
      if(width===320)await page.screenshot({path:path.join(out,'narrow-controls.png')});
      assert.deepEqual(errors,[],`${width}: reduced-motion browser errors`);
      report.push({width,reducedMotion:true,colors:true,cartridgeControls:true,layoutStable:true,errors});await page.close();
    }
    const saved=await pageFor(390,{reducedMotion:'no-preference'});
    await saved.addInitScript(()=>Object.defineProperty(navigator,'connection',{value:{saveData:true}}));
    await saved.goto(origin,{waitUntil:'networkidle'});await saved.locator('[data-ready=true][data-pose=open]').waitFor();
    assert(await saved.evaluate(()=>document.getElementById('hero-preview').paused));
    assert.equal(await saved.locator('.model-opening').getAttribute('src'),null);
    await saved.getByRole('button',{name:'Close lid',exact:true}).click();await saved.locator('[data-pose=closed]').waitFor();await saved.close();
    fs.writeFileSync(path.join(out,'checks.json'),JSON.stringify(report,null,2));
    console.log('PASS: fixed angle, stable first load and lid controls, matching opening/closing, all finishes and persistence, native cartridge endpoints, prominent wordmark, responsive layout and reduced motion');
  }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
