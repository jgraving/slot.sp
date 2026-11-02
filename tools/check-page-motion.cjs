// Exercise real document navigations and the browser's transition snapshots.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {execFileSync}=require('node:child_process');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'../site/public'),out=path.resolve(process.argv[2]);
fs.mkdirSync(out,{recursive:true});
const origin='https://jakegraving.com/slot.sp/';
const mime={'.js':'text/javascript','.css':'text/css','.html':'text/html','.wasm':'application/wasm','.woff2':'font/woff2','.svg':'image/svg+xml','.png':'image/png','.webp':'image/webp','.mp4':'video/mp4','.json':'application/json'};
(async()=>{
 const browser=await chromium.launch({executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']});
 const results=[];
 async function open(width,reducedMotion='no-preference'){
  const page=await browser.newPage({viewport:{width,height:900},reducedMotion,hasTouch:width===980});
  page.on('pageerror',error=>{throw error;});
  await page.route(origin+'**',async route=>{
   let file=path.resolve(root,decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
   if(!file.startsWith(root+path.sep)&&file!==root)return route.fulfill({status:403});
   if(fs.existsSync(file)&&fs.statSync(file).isDirectory())file=path.join(file,'index.html');
   if(!fs.existsSync(file))return route.fulfill({status:404});
   await route.fulfill({path:file,contentType:mime[path.extname(file)]||'application/octet-stream'});
  });
  await page.addInitScript(()=>{
   window.__pageReveals=0;
   addEventListener('pagereveal',event=>{
    window.__pageReveals++;
    const sample=window.__pageMotion={native:!!event.viewTransition,url:location.href};
    if(!event.viewTransition)return;
    event.viewTransition.finished.then(()=>{sample.finished=true;});
    event.viewTransition.ready.then(()=>{
     const animations=window.__pageAnimations=document.getAnimations().filter(a=>/^slot-page-(in|out)$/.test(a.animationName));
     animations.forEach(a=>{a.pause();a.currentTime=32;});
     const style=getComputedStyle(document.documentElement,'::view-transition-new(root)');
     Object.assign(sample,{ready:true,direction:document.documentElement.dataset.pageDirection,animation:style.animationName,x:new DOMMatrix(style.transform).m41,opacity:Number(style.opacity),headerAnimation:getComputedStyle(document.documentElement,'::view-transition-new(site-header)').animationName,count:animations.length});
    }).catch(error=>{sample.error=error.message;});
   });
  });
  return page;
 }
 async function transition(page,action,route,direction,label,capture=false){
  await action();
  await page.waitForURL(origin+route);
  await page.waitForFunction(()=>window.__pageMotion?.ready||window.__pageMotion?.error,{},{timeout:10000});
  const sample=await page.evaluate(()=>window.__pageMotion);
  assert(sample.native&&!sample.error,`${label}: native transition: ${JSON.stringify(sample)}`);
  assert.equal(sample.direction,direction,`${label}: history direction`);
  assert.equal(sample.animation,'slot-page-in');
  assert(sample.count===2&&sample.opacity>0&&sample.opacity<1,`${label}: both sliding layers are animated`);
  assert(direction==='back'?sample.x<0:sample.x>0,`${label}: incoming page slides from the correct side: ${sample.x}`);
  assert.equal(sample.headerAnimation,'none',`${label}: header stays anchored`);
  assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${label}: horizontal overflow`);
  if(!route){
   await page.locator('.sp-stage[data-ready=true]').waitFor();
   assert.notEqual(await page.locator('.sp-stage').getAttribute('data-pose'),'opening','Wait for the page slide before starting the lid intro');
  }
  if(capture){
   const frames=path.join(out,label);fs.mkdirSync(frames,{recursive:true});
   for(let i=0;i<=8;i++){
    await page.evaluate(time=>window.__pageAnimations.forEach(a=>{a.currentTime=time;}),i*40);
    await page.screenshot({path:path.join(frames,`${String(i).padStart(2,'0')}.png`)});
   }
   execFileSync('python',['-c',`from PIL import Image
from pathlib import Path
import sys
p=Path(sys.argv[1]);frames=[Image.open(f).convert('RGB') for f in sorted(p.glob('*.png'))]
frames[0].save(p.with_suffix('.gif'),save_all=True,append_images=frames[1:],duration=[500]+[40]*7+[700],loop=0,optimize=True)
`,frames]);
  }else await page.screenshot({path:path.join(out,`${label}.png`)});
  await page.evaluate(()=>window.__pageAnimations.forEach(a=>a.play()));
  await page.waitForFunction(()=>window.__pageMotion.finished&&document.documentElement.dataset.pageTransition!=='active');
  assert.equal(await page.locator('h1').count(),1,`${label}: destination rendered normally`);
  results.push({label,...sample});
 }
 try{
  for(const width of [1366,980,390]){
   const page=await open(width);
   await page.goto(origin+'installing/',{waitUntil:'networkidle'});
   assert(!await page.evaluate(()=>window.__pageMotion.native),'Initial loads have no page slide');
   await transition(page,()=>page.locator('header').getByRole('link',{name:'Update',exact:true}).click(),'updating/','forward',`${width}-forward`,width===980);
   await transition(page,()=>page.goBack({waitUntil:'domcontentloaded'}),'installing/','back',`${width}-back`);
   await transition(page,()=>page.goForward({waitUntil:'domcontentloaded'}),'updating/','forward',`${width}-history-forward`);
   await transition(page,()=>page.locator('header').getByRole('link',{name:'Cart Studio',exact:true}).click(),'studio/','forward',`${width}-studio`);
   await transition(page,()=>page.locator('header').getByRole('link',{name:'Guide',exact:true}).click(),'features/guide/','forward',`${width}-guide`);
   const reveals=await page.evaluate(()=>window.__pageReveals);
   await page.locator('.md-typeset a.headerlink[href="#find-a-feature"]').click();
   assert(new URL(page.url()).hash==='#find-a-feature');
   assert.equal(await page.evaluate(()=>window.__pageReveals),reveals,'Section anchors keep ordinary in-page navigation');
   await transition(page,()=>page.locator('header .mast-mark').click(),'','forward',`${width}-home`);
   await page.close();
  }
  const reduced=await open(980,'reduce');
  await reduced.goto(origin+'installing/',{waitUntil:'networkidle'});
  await reduced.locator('header').getByRole('link',{name:'Update',exact:true}).click();
  await reduced.waitForURL(origin+'updating/');
  await reduced.waitForFunction(()=>window.__pageMotion);
  assert(!await reduced.evaluate(()=>window.__pageMotion.native),'Reduced motion opts out of the page slide');
  await reduced.goBack({waitUntil:'networkidle'});
  assert(!await reduced.evaluate(()=>window.__pageMotion.native),'Reduced motion also applies to browser history');
  await reduced.close();
  fs.writeFileSync(path.join(out,'page-motion.json'),JSON.stringify(results,null,2));
  console.log('PASS: real page slides, anchored header, reverse Back, forward history, Studio, guides, anchors, delayed lid intro and reduced motion at desktop, Android desktop and phone widths');
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
