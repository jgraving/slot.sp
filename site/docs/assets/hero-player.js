// A small stateful player for native UI captures. The canvas holds the current
// frame during seeks, so unrelated frames from the recording never flash.
export function createHero(video, canvas, {media, reduced, onState}) {
  const context = canvas.getContext('2d', {alpha:false});
  const poster = document.querySelector('.hero-screen-poster');
  let timeline, atlas, color='platinum', generation=0, operation=0;
  let selected=0, target=0, seated=false, wantSeated=false;
  let active=false, paused=false, clip=null, raf=0, lastTime=-1;
  let bootPending=!reduced.matches && !navigator.connection?.saveData;
  let failed=false, disposed=false, changing=false, pendingColor=null, boots=0;
  let demo=bootPending, demoTimer=0, demoStep=0;
  const films=new Map();
  const animate=()=>!reduced.matches && !navigator.connection?.saveData;
  const wrap=index=>(index%5+5)%5;
  const url=name=>new URL(name,media).href;
  const manifest=fetch(url('timeline.json')).then(response=>{
    if(!response.ok)throw Error('Preview timeline unavailable');return response.json();
  });
  function state() {
    const moving=!!clip || bootPending;
    canvas.dataset.state=clip?.key==='boot'||bootPending?'boot':clip?clip.key.split('-')[0]:seated?'seated':'shelf';
    canvas.dataset.cart=String(wrap(selected));
    canvas.dataset.target=String(wrap(target));
    canvas.dataset.paused=String(paused||!active);
    canvas.dataset.failed=String(failed);
    canvas.setAttribute('aria-label',`${timeline?.carts[wrap(selected)]||'Cartridge'}${seated?' in the slot':' selected'}`);
    canvas.dataset.demo=String(demo);
    onState({moving,paused,active,failed,demo});
  }
  function painted() {canvas.dataset.painted='true';poster.hidden=true;canvas.dispatchEvent(new Event('hero:frame'));}
  function still(row) {
    if(!atlas)return;
    context.drawImage(atlas,0,row*480,720,480,0,0,720,480);painted();
  }
  function settled() {still((seated?6:1)+wrap(selected));}
  async function film() {
    const requested=color;
    if(!films.has(requested))films.set(requested,fetch(url(`${requested}.mp4`)).then(async response=>{
      if(!response.ok)throw Error('Preview actions unavailable');
      return URL.createObjectURL(await response.blob());
    }).catch(error=>{films.delete(requested);throw error;}));
    const source=await films.get(requested);
    if(requested!==color||disposed)return false;
    if(video.src!==source) {
      await new Promise((resolve,reject)=>{
        const done=()=>{cleanup();resolve();},error=()=>{cleanup();reject(Error('Preview video unavailable'));};
        const cleanup=()=>{video.removeEventListener('loadeddata',done);video.removeEventListener('error',error);};
        video.addEventListener('loadeddata',done,{once:true});video.addEventListener('error',error,{once:true});
        video.src=source;video.load();
      });
    }
    // Keep at most two compressed films; colors are fetched only when selected.
    for(const [key,value] of films)if(films.size>2&&key!==color){films.delete(key);value.then(URL.revokeObjectURL).catch(()=>{});}
    return true;
  }
  function stopFrames() {cancelAnimationFrame(raf);raf=0;video.pause();}
  function finish(current) {
    if(!clip||current!==operation)return;
    stopFrames();
    const completed=clip;clip=null;
    if(completed.key==='boot') {selected=0;seated=false;bootPending=false;}
    else if(completed.key.startsWith('insert'))seated=true;
    else if(completed.key.startsWith('eject'))seated=false;
    else selected=completed.destination;
    settled();state();
    if(pendingColor){const next=pendingColor;pendingColor=null;setColor(next).then(pump);}else pump();
  }
  function tick() {
    raf=0;
    if(!clip||!active||paused||disposed)return;
    if(!video.seeking&&video.readyState>=2) {
      if(video.currentTime>=clip.end/timeline.fps-0.002) {finish(clip.operation);return;}
      if(video.currentTime>=clip.start/timeline.fps-0.002&&video.currentTime!==lastTime) {
        context.drawImage(video,0,0,720,480);painted();lastTime=video.currentTime;
      }
    }
    raf=requestAnimationFrame(tick);
  }
  function resume() {
    if(!clip?.ready||!active||paused||disposed)return;
    // Shorten a burst of queued browsing without restarting its native motion.
    video.playbackRate=clip.key.startsWith('right')||clip.key.startsWith('left')?(Math.abs(target-selected)>1?1.7:1.2):1;
    video.play().then(()=>{if(!raf)raf=requestAnimationFrame(tick);}).catch(()=>{
      paused=true;state();
    });
  }
  async function start(key,destination=selected) {
    const current=++operation;
    if(key==='boot')canvas.dataset.boots=String(++boots);
    const [start,end]=timeline.segments[key];
    clip={key,start,end,destination,operation:current,ready:false};state();
    if(!animate()) {finish(current);return;}
    try {
      if(!await film()||current!==operation)return;
      await new Promise((resolve,reject)=>{
        const time=(start+0.05)/timeline.fps;
        if(!video.seeking&&Math.abs(video.currentTime-time)<0.0005){resolve();return;}
        const done=()=>{cleanup();resolve();},error=()=>{cleanup();reject(Error('Preview seek failed'));};
        const cleanup=()=>{video.removeEventListener('seeked',done);video.removeEventListener('error',error);};
        video.addEventListener('seeked',done,{once:true});video.addEventListener('error',error,{once:true});video.currentTime=time;
      });
      if(current!==operation||disposed)return;
      // Present the first decoded frame before its clock starts. The 3D
      // renderer may need to upload/compile the screen texture on first load;
      // that work must not consume the short native boot animation.
      context.drawImage(video,0,0,720,480);painted();
      await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
      if(current!==operation||disposed)return;
      clip.ready=true;lastTime=-1;resume();
    } catch(error) {
      // Static endpoints keep every control functional if decoding is unavailable.
      if(current===operation){console.warn('Native action playback unavailable.',error);failed=true;finish(current);}
    }
  }
  function pump() {
    if(!timeline||!atlas||clip||changing||disposed)return;
    if(bootPending) {if(active&&!paused)start('boot');return;}
    if(seated&&(!wantSeated||selected!==target))start(`eject-${wrap(selected)}`);
    else if(!seated&&selected!==target) {
      const step=Math.sign(target-selected);start(`${step>0?'right':'left'}-${wrap(selected)}`,selected+step);
    } else if(!seated&&wantSeated)start(`insert-${wrap(selected)}`);
    else {state();scheduleDemo();}
  }
  function clearDemoTimer() {clearTimeout(demoTimer);demoTimer=0;}
  function scheduleDemo() {
    clearDemoTimer();
    if(!demo||!animate()||!active||paused||clip||bootPending||changing||failed)return;
    demoTimer=setTimeout(()=>{
      demoTimer=0;
      if(!demo||!active||paused||disposed)return;
      // Native browsing and slot motion, with a readable pause at each stop.
      const action=seated?'eject':demoStep++%2===0?'right':'insert';
      input(action,false);
    },seated?1300:1000);
  }
  function takeControl() {
    if(!demo)return;
    demo=false;clearDemoTimer();state();
  }
  function input(action,manual=true) {
    if(disposed)return;
    if(manual)takeControl();
    clearDemoTimer();
    paused=false;
    if(action==='boot') {
      if(bootPending||clip?.key==='boot')return;
      ++operation;stopFrames();clip=null;selected=target=0;seated=wantSeated=false;
      bootPending=true;still(0);
    } else if(['left','right','up','down'].includes(action)) {
      // The device's D-pad does not eject a running game. This preview has one
      // GBA shelf; Up/Down skip to the next distinct initial, as the OS does.
      if(seated||wantSeated)return;
      const step=action==='right'||action==='down'?1:-1;
      let distance=1;
      if(action==='up'||action==='down') {
        const initial=timeline?.carts[wrap(target)]?.[0];
        while(distance<5&&timeline?.carts[wrap(target+step*distance)]?.[0]===initial)distance++;
      }
      target=Math.max(selected-5,Math.min(selected+5,target+step*distance));
    } else if(action==='insert')wantSeated=true;
    else if(action==='eject')wantSeated=false;
    state();if(clip)resume();else pump();
  }
  async function setColor(next) {
    next=next==='gold'?'platinum':next.replaceAll(' ','_');
    if(clip){pendingColor=next;return;}
    if(next===color&&atlas)return;
    color=next;const current=++generation;changing=true;
    // The old frame remains painted until the matching finish is decoded.
    ++operation;stopFrames();
    const image=new Image();image.src=url(`${next}.webp`);
    try {
      const [data]=await Promise.all([manifest,image.decode()]);
      if(current!==generation||disposed)return;
      timeline=data;atlas=image;failed=false;changing=false;canvas.dataset.finish=next;
      if(bootPending)still(0);else settled();state();pump();
    } catch {if(current===generation){failed=true;changing=false;bootPending=false;state();}}
  }
  const motionChange=()=>{
    if(reduced.matches){takeControl();bootPending=false;if(clip)finish(clip.operation);else{settled();state();pump();}}
  };
  reduced.addEventListener('change',motionChange);
  setColor(color);
  return {
    input,setColor,takeControl,
    setActive(value){if(active===value)return;active=value;if(!active){clearDemoTimer();stopFrames();}else if(clip)resume();else pump();state();},
    play(){demo=animate();paused=false;if(clip)resume();else pump();state();},
    toggle(){if((demo||clip||bootPending)&&!paused){takeControl();paused=true;stopFrames();}else{demo=animate();paused=false;if(clip)resume();else pump();}state();},
    dispose(){disposed=true;++operation;clearDemoTimer();stopFrames();reduced.removeEventListener('change',motionChange);for(const value of films.values())value.then(URL.revokeObjectURL).catch(()=>{});}
  };
}
