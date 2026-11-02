// Capture the production Three.js model for the README and non-WebGL opening.
const fs=require('node:fs'),path=require('node:path'),http=require('node:http'),crypto=require('node:crypto');
const {execFileSync}=require('node:child_process');
const {chromium}=require('playwright');
const repo=path.resolve(__dirname,'..'),output=path.resolve(process.argv[2]);
const videoPath=path.resolve(process.argv[3]||path.join(repo,'previews/homepage/hero-light-platinum.mp4'));
fs.mkdirSync(path.join(output,'model-frames'),{recursive:true});
execFileSync('python3',[path.join(repo,'previews/extract_model_frames.py'),videoPath,output],{stdio:'inherit'});
const native=JSON.parse(fs.readFileSync(path.join(output,'native-frames.json')));
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const modelHash=hash(path.join(repo,'site/docs/assets/sp-model.js'));
const model=`<figure class="sp-preview"><div class="sp-stage"><div class="sp-render-fallback"><video id="hero-preview" muted playsinline preload="none"></video></div></div><div class="model-controls"><button class="model-fold"></button><div class="model-angle"><input type="range" min="0" max="180"><output></output></div></div></figure>`;
function html(readme){return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;overflow:hidden;background:${readme?'#e8e3d8':'#0e1423'}}
@font-face{font-family:ReadmeSans;src:url('/crates/slot-ui/assets/fonts/display.ttf') format('truetype')}
.sp-preview{margin:0;position:absolute;left:${readme?'500':'0'}px;top:${readme?'60':'0'}px;width:480px;height:600px}.sp-stage{position:relative;width:480px;height:600px}
.sp-model-canvas{position:absolute;inset:0;width:100%;height:100%}.sp-render-fallback,.hero-screen,.model-controls,#hero-preview{display:none}
.readme-brand{position:absolute;left:58px;top:50%;transform:translateY(-50%);width:400px}.readme-brand img{display:block;width:365px;height:auto;filter:brightness(0) saturate(100%) invert(12%) sepia(15%) saturate(1000%) hue-rotate(155deg)}.readme-brand p{margin:18px 0 0;font:34px/1.3 ReadmeSans,sans-serif;letter-spacing:-.025em;color:#1c2a31}.readme-brand p span{color:#0b6e61}
</style></head><body>${readme?'<div class="readme-brand"><img src="/site/docs/assets/wordmark.png" alt="slot.sp"><p>Your games.<br><span>SP style.</span></p></div>':''}${model}<script type="module">import {mountSP} from '/site/docs/assets/sp-model.js';window.captureSP=mountSP(document.querySelector('.sp-stage'),document.querySelector('video'),{capture:true});window.captureReady=true;</script></body></html>`;}
const server=http.createServer((req,res)=>{
 const url=new URL(req.url,'http://localhost');
 if(url.pathname==='/'){res.setHeader('Content-Type','text/html');return res.end(html(url.searchParams.has('readme')));}
 const file=/^\/native-frames\/\d{4}\.png$/.test(url.pathname)?path.join(output,url.pathname.slice(1)):path.resolve(repo,'.'+url.pathname);
 if(!file.startsWith(repo+path.sep)&&!file.startsWith(path.join(output,'native-frames')+path.sep)){res.writeHead(403);return res.end();}
 try {const data=fs.readFileSync(file);res.setHeader('Content-Type',({'.js':'text/javascript','.png':'image/png','.css':'text/css','.ttf':'font/ttf'})[path.extname(file)]||'application/octet-stream');res.end(data);}catch{res.writeHead(404);res.end();}
});
(async()=>{
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 const origin=`http://127.0.0.1:${server.address().port}`;
 const browser=await chromium.launch({executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 const errors=[];
 try {
  const page=await browser.newPage({viewport:{width:480,height:600},deviceScaleFactor:1.5,reducedMotion:'reduce'});
  page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto(origin);await page.waitForFunction(()=>window.captureReady,{},{timeout:60000});
  let projection;
  for(const color of ['platinum','graphite','pearl blue','pearl pink','gold']) {
   const name=color.replaceAll(' ','_'),directory=path.join(output,'opening',name);fs.mkdirSync(directory,{recursive:true});
   for(let frame=0;frame<33;frame++) {
    const t=Math.max(0,Math.min(1,(frame/30-.08)/.9)),lidAngle=150*(1-(1-t)**3);
    projection=await page.evaluate(args=>window.captureSP.captureFrame(args),{color,lidAngle});
    await page.screenshot({path:path.join(directory,`${String(frame).padStart(4,'0')}.png`)});
   }
   await page.evaluate(()=>{document.documentElement.style.background='transparent';document.body.style.background='transparent';});
   for(const [pose,lidAngle]of [['open',150],['closed',0]]) {
    await page.evaluate(args=>window.captureSP.captureFrame(args),{color,lidAngle});
    await page.screenshot({path:path.join(output,`${name}-${pose}.png`),omitBackground:true});
   }
   await page.evaluate(()=>{document.documentElement.style.background='#0e1423';document.body.style.background='#0e1423';});
   console.log('Captured 3D opening and posters:',color);
  }
  fs.writeFileSync(path.join(output,'projection.json'),JSON.stringify(projection,null,2));
  await page.close();
  const readme=await browser.newPage({viewport:{width:1000,height:720},deviceScaleFactor:1,reducedMotion:'reduce'});
  readme.on('pageerror',e=>errors.push(e.message));
  await readme.goto(origin+'/?readme');await readme.waitForFunction(()=>window.captureReady,{},{timeout:60000});await readme.evaluate(()=>document.fonts.ready);
  let index=0;
  async function frame(lidAngle,source=null){await readme.evaluate(args=>window.captureSP.captureFrame(args),{lidAngle,color:'platinum',source});await readme.screenshot({path:path.join(output,'model-frames',`${String(index++).padStart(4,'0')}.png`)});}
  for(let i=0;i<15;i++){const t=Math.min(1,Math.max(0,(i*.08-.08)/.9));await frame(150*(1-(1-t)**3));}
  const poster_index=index;
  for(let i=0;i<native.frames;i++)await frame(150,`${origin}/native-frames/${String(i).padStart(4,'0')}.png`);
  for(let i=0;i<12;i++){const t=(i+1)/12;await frame(150*(1-(3*t*t-2*t*t*t)));}
  for(let i=0;i<4;i++)await frame(0);
  if(errors.length)throw Error(errors.join('\n'));
  fs.writeFileSync(path.join(output,'model-capture.json'),JSON.stringify({frames:index,frame_ms:80,duration:index*.08,poster_index,model_sha256:modelHash,browser_version:browser.version(),logo_sha256:hash(path.join(repo,'site/docs/assets/wordmark.png')),logo_font_sha256:hash(path.join(repo,'crates/slot-ui/assets/fonts/display.ttf')),logo_font_profile:'display',native_sha256:hash(videoPath)},null,2));
  console.log(`Captured ${index} README frames using the production 3D model`);
 }finally{await browser.close();server.close();}
})().catch(error=>{console.error(error);server.close();process.exitCode=1;});
