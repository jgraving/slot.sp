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
 async function open(width,reducedMotion='no-preference',fallback=false){
  const page=await browser.newPage({viewport:{width,height:900},reducedMotion,hasTouch:width===980});
  page.on('pageerror',error=>{throw error;});
  await page.route(origin+'**',async route=>{
   let file=path.resolve(root,decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
   if(!file.startsWith(root+path.sep)&&file!==root)return route.fulfill({status:403});
   if(fs.existsSync(file)&&fs.statSync(file).isDirectory())file=path.join(file,'index.html');
   if(!fs.existsSync(file))return route.fulfill({status:404});
   await route.fulfill({path:file,contentType:mime[path.extname(file)]||'application/octet-stream'});
  });
  await page.addInitScript(fallback=>{
   if(fallback){
    for(const name of ['onpageswap','onpagereveal']){
     for(let owner=window;owner;owner=Object.getPrototypeOf(owner)){
      if(Object.prototype.hasOwnProperty.call(owner,name))delete owner[name];
     }
    }
    if('onpagereveal' in window)throw Error('Cross-document capability was not disabled for the fallback test');
    const capture=()=>{
     if(document.documentElement.dataset.pageFallback!=='incoming')return;
     requestAnimationFrame(()=>{
      const animations=window.__fallbackAnimations=document.getAnimations().filter(a=>a.animationName==='slot-fallback-in');
      if(sessionStorage.getItem('slot-test-live-film')!=='1')animations.forEach(a=>{a.pause();a.currentTime=32;});
      const content=document.querySelector('.md-container,body>.wrap'),style=getComputedStyle(content);
      window.__fallbackMotion={ready:true,direction:document.documentElement.dataset.pageDirection,x:new DOMMatrix(style.transform).m41,opacity:Number(style.opacity),count:animations.length};
     });
    };
    addEventListener('DOMContentLoaded',capture);
    addEventListener('pageshow',event=>{if(event.persisted)capture();});
   }
   window.__pageReveals=0;
   addEventListener('pagereveal',event=>{
    window.__pageReveals++;
    const sample=window.__pageMotion={native:!!event.viewTransition,url:location.href};
    if(!event.viewTransition)return;
    event.viewTransition.finished.then(()=>{sample.finished=true;});
    event.viewTransition.ready.then(()=>{
     const animations=window.__pageAnimations=document.getAnimations().filter(a=>/^slot-page-/.test(a.animationName)||a.animationName?.includes('view-transition'));
     animations.forEach(a=>{a.pause();a.currentTime=32;});
     const style=getComputedStyle(document.documentElement,'::view-transition-new(root)');
     const pill=getComputedStyle(document.documentElement,'::view-transition-group(site-nav-pill)');
     const surface=getComputedStyle(document.documentElement,'::view-transition-new(site-nav-pill)');
     Object.assign(sample,{ready:true,direction:document.documentElement.dataset.pageDirection,animation:style.animationName,x:new DOMMatrix(style.transform).m41,opacity:Number(style.opacity),headerAnimation:getComputedStyle(document.documentElement,'::view-transition-new(site-header)').animationName,pillAnimation:pill.animationName,pillDuration:pill.animationDuration,pillOpacity:surface.opacity,count:animations.length});
    }).catch(error=>{sample.error=error.message;});
   });
  },fallback);
  return page;
 }
 async function fallbackTransition(page,action,route,direction,label,film=false){
  let recording;
  if(film){
   await page.evaluate(()=>sessionStorage.setItem('slot-test-live-film','1'));
   const folder=path.join(out,label+'-film');fs.mkdirSync(folder,{recursive:true});
   const cdp=await page.context().newCDPSession(page),frames=[];
   cdp.on('Page.screencastFrame',frame=>{
    const file=path.join(folder,String(frames.length).padStart(4,'0')+'.jpg');
    fs.writeFileSync(file,Buffer.from(frame.data,'base64'));frames.push({file,time:frame.metadata.timestamp});
    cdp.send('Page.screencastFrameAck',{sessionId:frame.sessionId}).catch(()=>{});
   });
   await cdp.send('Page.startScreencast',{format:'jpeg',quality:85,maxWidth:980,maxHeight:900,everyNthFrame:1});
   recording={cdp,frames,folder};
  }
  await action();await page.waitForURL(origin+route);
  await page.waitForFunction(()=>window.__fallbackMotion?.ready);
  const sample=await page.evaluate(()=>window.__fallbackMotion);
  assert.equal(await page.locator('html').getAttribute('data-page-motion'),'fallback');
  assert.equal(sample.direction,direction,label+': fallback direction');
  assert(sample.count>0,label+': older browsers must receive an actual slide');
  assert.equal(sample.opacity,1,label+': page remains opaque');
  assert(direction==='back'?sample.x<0:sample.x>0,label+': entry direction');
  const positions=[];
  let lastTime=0;
  for(const time of [0,100,300,650]){
   if(film&&time>lastTime)await page.waitForTimeout(time-lastTime);
   lastTime=time;
   const position=await page.evaluate(({time,film})=>{
    if(!film)window.__fallbackAnimations.forEach(a=>a.currentTime=time);
    const style=getComputedStyle(document.querySelector('.md-container,body>.wrap'));
    const header=document.querySelector('.md-header,.mast'),pill=header.querySelector('.selector-surface');
    return {x:new DOMMatrix(style.transform).m41,opacity:Number(style.opacity),headerX:header.getBoundingClientRect().x,scrollX:window.scrollX,pillOpacity:getComputedStyle(pill).opacity};
   },{time,film});
   assert.equal(position.opacity,1,label+': no transparent content');
   assert.equal(position.headerX,0,label+': header must not slide with the content');
   if(route)assert.equal(position.pillOpacity,'1',label+': pill stays visible on the new document');
   positions.push(position);
  }
  assert(Math.abs(film?sample.x:positions[0].x)>=48&&Math.abs(positions.at(-1).x)<1,label+': actual spatial travel');
  await page.screenshot({path:path.join(out,label+'.png')});
  if(!film)await page.evaluate(()=>window.__fallbackAnimations.forEach(a=>{a.currentTime=0;a.play();}));
  await page.waitForFunction(()=>document.documentElement.dataset.pageTransition!=='active');
  if(recording){
   await page.evaluate(()=>sessionStorage.removeItem('slot-test-live-film'));
   await recording.cdp.send('Page.stopScreencast');await recording.cdp.detach();
   fs.writeFileSync(path.join(recording.folder,'frames.json'),JSON.stringify(recording.frames));
   execFileSync('python3',['-c',`from PIL import Image
from pathlib import Path
import sys,json
p=Path(sys.argv[1]);data=json.loads((p/'frames.json').read_text())
images=[Image.open(frame['file']).convert('RGB') for frame in data]
durations=[max(20,min(200,round((b['time']-a['time'])*1000))) for a,b in zip(data,data[1:])]+[600]
assert len(images)>8, ('Need real animation frames',len(images))
images[0].save(p.with_suffix('.gif'),save_all=True,append_images=images[1:],duration=durations,loop=0,optimize=True)
`,recording.folder]);
  }
  assert.equal(await page.locator('h1').count(),1);
  results.push({label,fallback:true,...sample,positions});
 }
 async function transition(page,action,route,direction,label,capture=false){
  // Start each navigation from the active destination rather than a lingering
  // mouse hover left behind by the previous scripted click/history action.
  await page.mouse.move(8,850);
  await page.waitForFunction(()=>{
   const root=document.querySelector('header .slot-selector'),active=root.querySelector('a[aria-current]'),pill=root.querySelector('.selector-surface');
   if(!active)return getComputedStyle(pill).opacity==='0';
   const a=active.getBoundingClientRect(),p=pill.getBoundingClientRect();
   return Math.abs(a.x-p.x)<.2&&Math.abs(a.y-p.y)<.2&&Math.abs(a.width-p.width)<.2;
  });
  await action();
  await page.waitForURL(origin+route);
  await page.waitForFunction(()=>window.__pageMotion?.ready||window.__pageMotion?.error,{},{timeout:10000});
  const sample=await page.evaluate(()=>window.__pageMotion);
  assert(sample.native&&!sample.error,`${label}: native transition: ${JSON.stringify(sample)}`);
  assert.equal(sample.direction,direction,`${label}: history direction`);
  assert(sample.animation.includes('slot-page-in'));
  assert(sample.count>=2,`${label}: both page layers slide`);
  assert.equal(sample.opacity,1,`${label}: the page and handheld must stay opaque throughout the slide`);
  assert(direction==='back'?sample.x<0:sample.x>0,`${label}: incoming page slides from the correct side: ${sample.x}`);
  assert.equal(sample.headerAnimation,'none',`${label}: header stays anchored`);
  if(route){
   assert.notEqual(sample.pillAnimation,'none',`${label}: one shared pill must animate between its captured positions`);
   assert.equal(sample.pillDuration,'0.42s',`${label}: expressive fast spatial spring`);
   assert.equal(sample.pillOpacity,'1',`${label}: the shared pill never fades away`);
   const positions=[];
   for(const time of [0,100,210,420]){
    const position=await page.evaluate(time=>{
     window.__pageAnimations.forEach(a=>{a.currentTime=time;});
     const group=getComputedStyle(document.documentElement,'::view-transition-group(site-nav-pill)');
     const surface=getComputedStyle(document.documentElement,'::view-transition-new(site-nav-pill)');
     return {x:new DOMMatrix(group.transform).m41,y:new DOMMatrix(group.transform).m42,width:parseFloat(group.width),height:parseFloat(group.height),opacity:surface.opacity};
    },time);
    assert.equal(position.opacity,'1',`${label}: pill opacity at ${time}ms`);
    assert(position.width>20&&position.height>=44,`${label}: pill retains its shape throughout travel`);
    const headerHeight=await page.locator('.md-header,.mast').evaluate(el=>Math.ceil(el.getBoundingClientRect().height));
    const png=await page.screenshot({path:path.join(out,`${label}-pill-${time}.png`),clip:{x:0,y:0,width:page.viewportSize().width,height:headerHeight}});
    execFileSync('python3',['-c',`import io,json,sys
from PIL import Image
im=Image.open(io.BytesIO(sys.stdin.buffer.read())).convert('RGB');p=json.loads(sys.argv[1])
box=(round(p['x']),round(p['y']),round(p['x']+p['width']),round(p['y']+p['height']))
pixels=list(im.crop(box).get_flattened_data())
surface=sum(max(abs(a-b) for a,b in zip(pixel,(51,79,82)))<5 for pixel in pixels)
# Rounded corners and text interrupt the fill; a disappearing pill cannot
# retain most of its solid surface. Avoid sampling an antialiased letter.
assert surface>len(pixels)*.45, ('Pill disappeared in rendered pixels',surface,len(pixels),sys.argv[2])
if sys.argv[3]=='420':
 ink=sum(r>130 and g>150 and b>150 for r,g,b in pixels)
 assert ink>12, ('Shared pill covers its label',ink,sys.argv[2])
`,JSON.stringify(position),label,String(time)],{input:png});
    positions.push(position);
   }
   assert(Math.abs(positions[0].x-positions.at(-1).x)>2||Math.abs(positions[0].y-positions.at(-1).y)>2,`${label}: indicator must travel, not be recreated at the destination: ${JSON.stringify(positions)}`);
   sample.pillPositions=positions;
   await page.evaluate(()=>window.__pageAnimations.forEach(a=>{a.currentTime=32;}));
  }
  assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${label}: horizontal overflow`);
  if(!route){
   await page.locator('.sp-stage[data-ready=true]').waitFor();
   assert.notEqual(await page.locator('.sp-stage').getAttribute('data-pose'),'opening','Wait for the page slide before starting the lid intro');
  }
  if(capture){
   const frames=path.join(out,label);fs.mkdirSync(frames,{recursive:true});
   for(let i=0;i<=13;i++){
    await page.evaluate(time=>window.__pageAnimations.forEach(a=>{a.currentTime=time;}),i*50);
    await page.screenshot({path:path.join(frames,`${String(i).padStart(2,'0')}.png`)});
   }
   execFileSync('python',['-c',`from PIL import Image
from pathlib import Path
import sys
p=Path(sys.argv[1]);frames=[Image.open(f).convert('RGB') for f in sorted(p.glob('*.png'))]
frames[0].save(p.with_suffix('.gif'),save_all=True,append_images=frames[1:],duration=[500]+[50]*12+[700],loop=0,optimize=True)
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
   await transition(page,()=>page.locator('header').getByRole('link',{name:'Cart Studio',exact:true}).click(),'studio/','forward',`${width}-studio`,width===980);
   await transition(page,()=>page.locator('header').getByRole('link',{name:'About',exact:true}).click(),'questions/','forward',`${width}-about`);
   await transition(page,()=>page.locator('header').getByRole('link',{name:'Guide',exact:true}).click(),'features/guide/','back',`${width}-guide`);
   const reveals=await page.evaluate(()=>window.__pageReveals);
   await page.locator('.md-typeset a.headerlink[href="#find-a-feature"]').click();
   assert(new URL(page.url()).hash==='#find-a-feature');
   assert.equal(await page.evaluate(()=>window.__pageReveals),reveals,'Section anchors keep ordinary in-page navigation');
   await transition(page,()=>page.locator('header .mast-mark').click(),'','back',`${width}-home`);
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
  for(const width of [1366,980,390]){
   const page=await open(width,'no-preference',true);
   await page.goto(origin+'installing/',{waitUntil:'networkidle'});
   assert.equal(await page.locator('html').getAttribute('data-page-fallback'),null,'Initial fallback load does not slide');
   await fallbackTransition(page,()=>page.locator('header').getByRole('link',{name:'Update',exact:true}).click(),'updating/','forward',`${width}-fallback-forward`,width===980);
   await fallbackTransition(page,()=>page.goBack({waitUntil:'domcontentloaded'}),'installing/','back',`${width}-fallback-back`);
   await fallbackTransition(page,()=>page.goForward({waitUntil:'domcontentloaded'}),'updating/','forward',`${width}-fallback-history-forward`);
   await fallbackTransition(page,()=>page.locator('header').getByRole('link',{name:'Cart Studio',exact:true}).click(),'studio/','forward',`${width}-fallback-studio`,width===980);
   await fallbackTransition(page,()=>page.locator('header').getByRole('link',{name:'About',exact:true}).click(),'questions/','forward',`${width}-fallback-about`);
   await fallbackTransition(page,()=>page.locator('header').getByRole('link',{name:'Guide',exact:true}).click(),'features/guide/','back',`${width}-fallback-guide`);
   const before=await page.evaluate(()=>window.__pageReveals);
   await page.locator('.md-typeset a.headerlink[href="#find-a-feature"]').click();
   assert.equal(await page.evaluate(()=>window.__pageReveals),before,'Fallback section anchors do not navigate');
   await fallbackTransition(page,()=>page.locator('header .mast-mark').click(),'','back',`${width}-fallback-home`);
   await page.close();
  }
  const still=await open(980,'reduce',true);
  await still.goto(origin+'installing/',{waitUntil:'networkidle'});
  await still.locator('header').getByRole('link',{name:'Update',exact:true}).click();await still.waitForURL(origin+'updating/');
  assert.equal(await still.locator('html').getAttribute('data-page-fallback'),null,'Fallback also respects reduced motion');
  await still.close();
  fs.writeFileSync(path.join(out,'page-motion.json'),JSON.stringify(results,null,2));
  console.log('PASS: 42 opaque native/fallback page slides; anchored header, visible pill, history, anchors, Studio, delayed lid and reduced motion; real-time fallback films');
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
