/* Reference-built RG SP. Coordinates are millimetre-scale; +Z faces the player.
 * The supplied opening clips guide the silhouette, recesses, hinge and finish.
 * This is a handcrafted reconstruction, not a photogrammetry scan. */
import * as T from './vendor/three-r180.min.js';

export function mountSP(stage, video) {
  if (!stage || stage.dataset.modelReady) return;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const renderer = new T.WebGLRenderer({alpha:true, antialias:true, powerPreference:'low-power'});
  renderer.setPixelRatio(Math.min(devicePixelRatio, 1.75));
  renderer.outputColorSpace = T.SRGBColorSpace;
  renderer.toneMapping = T.ACESFilmicToneMapping;
  renderer.toneMappingExposure = .95;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = T.PCFSoftShadowMap;
  renderer.shadowMap.autoUpdate = false;
  const canvas = renderer.domElement;
  canvas.className = 'sp-model-canvas';
  canvas.setAttribute('aria-hidden', 'true');
  const scene = new T.Scene();
  const camera = new T.PerspectiveCamera(32, 1, .1, 1500);
  camera.position.set(0, -2, 355);
  camera.lookAt(0, -2, 0);
  const product = new T.Group();
  scene.add(product);
  const lid = new T.Group();
  lid.position.set(0, 0, 1.0);
  product.add(lid);
  const controls = new Map();
  const disposables = new Set();
  const track = value => { disposables.add(value); return value; };
  const material = settings => track(new T.MeshPhysicalMaterial(settings));

  // Broad photographic softboxes, baked once into the reflection environment.
  const studio = new T.Scene();
  studio.background = new T.Color('#858b91');
  const room = new T.Mesh(new T.BoxGeometry(900,900,900),new T.MeshBasicMaterial({color:'#727980',side:T.BackSide}));
  studio.add(room);
  function softbox(w,h,x,y,z,strength) {
    const panel = new T.Mesh(new T.PlaneGeometry(w,h),new T.MeshBasicMaterial({color:new T.Color(strength,strength,strength)}));
    panel.position.set(x,y,z); panel.lookAt(0,0,0); studio.add(panel);
  }
  softbox(240,500,-260,180,190,6);
  softbox(95,420,280,40,160,4);
  softbox(420,90,0,320,-90,5);
  const pmrem = new T.PMREMGenerator(renderer);
  const environment = track(pmrem.fromScene(studio,.06));
  scene.environment = environment.texture;
  pmrem.dispose();
  studio.traverse(object => { object.geometry?.dispose(); object.material?.dispose(); });
  scene.add(new T.HemisphereLight(0xf4f7ff,0x616775,.85));
  const key = new T.DirectionalLight(0xfff9ee,2.6); key.position.set(-80,140,220); scene.add(key);
  key.castShadow=true;key.shadow.mapSize.set(1024,1024);
  Object.assign(key.shadow.camera,{left:-105,right:105,top:115,bottom:-110,near:50,far:500});
  key.shadow.normalBias=.12;key.shadow.bias=-.0001;
  const rim = new T.DirectionalLight(0xddeaff,1.1); rim.position.set(140,40,-60); scene.add(rim);

  function textureCanvas(width,height,paint) {
    const c=document.createElement('canvas'); c.width=width; c.height=height;
    paint(c.getContext('2d'),width,height);
    const texture=track(new T.CanvasTexture(c)); texture.colorSpace=T.SRGBColorSpace;
    texture.anisotropy=Math.min(4,renderer.capabilities.getMaxAnisotropy()); return texture;
  }
  const grain=textureCanvas(256,256,(ctx,w,h) => {
    let seed=34; const pixels=ctx.createImageData(w,h);
    for(let i=0;i<pixels.data.length;i+=4) {
      seed=(Math.imul(seed,1664525)+1013904223)>>>0;
      const value=160+(seed>>>27); pixels.data.set([value,value,value,255],i);
    }
    ctx.putImageData(pixels,0,0);
  });
  grain.colorSpace=T.NoColorSpace; grain.wrapS=grain.wrapT=T.RepeatWrapping; grain.repeat.set(.12,.12);
  const shell=material({color:0xaeb4b8,metalness:.62,roughness:.42,bumpMap:grain,bumpScale:.028,clearcoat:.14,clearcoatRoughness:.42,envMapIntensity:1.0});
  const edge=material({color:0xa1a8ae,metalness:.5,roughness:.4,bumpMap:grain,bumpScale:.02});
  const button=material({color:0x686f71,metalness:.12,roughness:.48,clearcoat:.1});
  const rubber=material({color:0x8b9295,metalness:0,roughness:.8});
  const dark=material({color:0x161b20,metalness:.12,roughness:.63});
  const seam=material({color:0x5d656d,metalness:.32,roughness:.6});
  const bezel=track(new T.MeshBasicMaterial({color:0x080b10,toneMapped:false}));
  const chrome=material({color:0xaab0b5,metalness:.8,roughness:.27});

  function rr(w,h,r,cx=0,cy=0,Type=T.Shape) {
    const s=new Type(), x=cx-w/2,y=cy-h/2;
    s.moveTo(x+r,y);s.lineTo(x+w-r,y);s.quadraticCurveTo(x+w,y,x+w,y+r);
    s.lineTo(x+w,y+h-r);s.quadraticCurveTo(x+w,y+h,x+w-r,y+h);
    s.lineTo(x+r,y+h);s.quadraticCurveTo(x,y+h,x,y+h-r);
    s.lineTo(x,y+r);s.quadraticCurveTo(x,y,x+r,y); return s;
  }
  function circleHole(shape,x,y,r) { const p=new T.Path(); p.absarc(x,y,r,0,Math.PI*2,true);shape.holes.push(p); }
  function mesh(geometry,mat,parent,x=0,y=0,z=0) {
    track(geometry);const item=new T.Mesh(geometry,mat);item.position.set(x,y,z);item.castShadow=!mat.transparent;item.receiveShadow=true;parent.add(item);return item;
  }
  function extrusion(shape,depth,bevelSize) {
    return new T.ExtrudeGeometry(shape,{depth:depth-2*bevelSize,bevelEnabled:bevelSize>0,bevelSize,bevelThickness:bevelSize,bevelSegments:4,steps:1,curveSegments:24});
  }
  function slab(parent,w,h,depth,radius,bevelSize,mat,x,y,front) {
    return mesh(extrusion(rr(w-2*bevelSize,h-2*bevelSize,radius-bevelSize),depth,bevelSize),mat,parent,x,y,front-depth+bevelSize);
  }
  function disc(parent,r,depth,mat,x,y,z) {
    const g=new T.CylinderGeometry(r,r,depth,64);g.rotateX(Math.PI/2);
    return mesh(g,mat,parent,x,y,z-depth/2);
  }
  // A revolved profile makes the recess a real bevel, not a painted circle.
  function ring(parent,profile,mat,x,y) {
    const g=new T.LatheGeometry(profile.map(([radius,z])=>new T.Vector2(radius,z)),80);g.rotateX(Math.PI/2);
    return mesh(g,mat,parent,x,y,0);
  }
  function engraving(text,parent,x,y,z,w,h,color='#42494d') {
    const single=text.length===1;
    const map=textureCanvas(single?256:512,single?256:128,(ctx,cw,ch) => {
      ctx.textAlign='center';ctx.textBaseline='middle';ctx.font=`500 ${single?176:98}px Arial, sans-serif`;
      ctx.fillStyle='rgba(255,255,255,.38)';ctx.fillText(text,cw/2+1.5,ch/2+2.5);
      ctx.fillStyle=color;ctx.fillText(text,cw/2,ch/2);
    });
    return mesh(new T.PlaneGeometry(w,h),track(new T.MeshBasicMaterial({map,transparent:true,depthWrite:false,toneMapped:false})),parent,x,y,z);
  }

  // Two separately bevelled mouldings with a fine, continuous parting seam.
  slab(product,82,83,10,5.2,1.3,edge,0,-43.5,-6.15);
  slab(product,81.9,82.8,.24,5.0,.04,seam,0,-43.5,-5.95);
  const face=rr(80.6,81.3,4.0,0,-43.5);
  const wells=[[-21,-32.5,14.0],[21,-32.5,14.0],[0,-15.5,3.5],[-5.4,-76,3.2],[5.4,-76,3.2]];
  wells.forEach(([x,y,r])=>circleHole(face,x,y,r));
  // The perforated skin has a square bore; a bevel wider than these holes
  // would fold their walls across each other. Keep its outer moulding separate.
  mesh(extrusion(face.clone(),5.85,.65),shell,product,0,0,-5.35);
  const speaker=[];
  for(let row=-2;row<=2;row++)for(let col=-2;col<=2;col++) {
    if(Math.abs(row)+Math.abs(col)>3)continue;
    const x=col*2.8, y=-59.5+row*2.8, r=.27;
    speaker.push([x,y]);circleHole(face,x,y,r);
  }
  mesh(extrusion(face,.18,0),shell,product,0,0,-.18);
  for(const [x,y,r] of wells) {
    disc(product,r-.15,.25,shell,x,y,-1.4);
    ring(product,[[r-.95,-1.4],[r-.85,-1.25],[r-.25,-.2],[r,.02]],shell,x,y);
    ring(product,[[r-.97,-1.45],[r-.95,-1.38]],seam,x,y);
  }
  for(const [x,y] of speaker)disc(product,.265,.1,dark,x,y,-.13);

  function roundButton(x,y,r,label,action) {
    disc(product,r+.25,.18,dark,x,y,-1.2);
    const group=new T.Group(); group.position.set(x,y,-.85);product.add(group);
    ring(group,[[0,0],[r-.3,0],[r,.2],[r,.85],[r-.17,1.12],[r-.48,1.25],[0,1.3]],button,0,0);
    if(label)engraving(label,group,0,0,1.34,r*1.18,r*1.18);
    if(action)controls.set(action,group);
    return group;
  }
  roundButton(21,-24.3,3.65,'X');
  roundButton(12.8,-32.5,3.65,'Y');
  roundButton(29.2,-32.5,3.65,'A','insert');
  roundButton(21,-40.7,3.65,'B','eject');
  roundButton(0,-15.5,2.25,'M','boot');
  roundButton(-5.4,-76,2.15,''); roundButton(5.4,-76,2.15,'');
  engraving('SELECT',product,-5.4,-70.3,.1,8,2.3);
  engraving('START',product,5.4,-70.3,.1,7.1,2.3);
  const cross=new T.Shape();
  const points=[[-3.4,10.5],[3.4,10.5],[3.4,3.4],[10.5,3.4],[10.5,-3.4],[3.4,-3.4],[3.4,-10.5],[-3.4,-10.5],[-3.4,-3.4],[-10.5,-3.4],[-10.5,3.4],[-3.4,3.4]];
  // Round each corner in the cross outline as well as the top edge.
  points.forEach(([x,y],i)=>{
    const prev=points[(i+11)%12],next=points[(i+1)%12],r=.65;
    const a=new T.Vector2(prev[0]-x,prev[1]-y).normalize().multiplyScalar(r);
    const b=new T.Vector2(next[0]-x,next[1]-y).normalize().multiplyScalar(r);
    if(i===0)cross.moveTo(x+a.x,y+a.y);else cross.lineTo(x+a.x,y+a.y);
    cross.quadraticCurveTo(x,y,x+b.x,y+b.y);
  });cross.closePath();
  const dpad=mesh(extrusion(cross,2.7,.38),button,product,-21,-32.5,-1.1);
  disc(product,1.5,.04,material({color:0x82898d,roughness:.75}),-21,-32.5,1.25);
  for(const [action,x]of [['left',-28],['right',-14]]) {
    const anchor=new T.Object3D();anchor.position.set(x,-32.5,1.8);product.add(anchor);controls.set(action,anchor);
  }

  // Hinge barrel: narrow end caps, two long knuckles and visible joint gaps.
  const segments=[[-38.5,4.8],[-19.5,32.4],[16.9,39.6],[39,3.3]];
  for(const [x,length]of segments) {
    const g=new T.CylinderGeometry(3.4,3.4,length,64,1);g.rotateZ(Math.PI/2);
    mesh(g,shell,product,x,-.4,-.8);
    const end=new T.CylinderGeometry(2.8,2.8,.18,48);end.rotateZ(Math.PI/2);
    mesh(end,edge,product,x-length/2-.05,-.4,-.8);
  }
  // Small shoulders connect the hinge to the deck; no floating lid plane.
  slab(product,7,10,9,2.8,.7,shell,-36,-5,-.4);
  slab(product,7,10,9,2.8,.7,shell,36,-5,-.4);

  // The photographed glass is 4:3. The existing 3:2 native capture is contained,
  // so neither cartridge proportions nor button hints are stretched to fit it.
  slab(lid,81,76,5.7,3.8,.75,edge,0,40.9,-.2);
  slab(lid,80.9,75.9,.20,3.8,.04,seam,0,40.9,-.1);
  const lidFace=rr(79.8,74.8,3.0,0,40.9);
  lidFace.holes.push(rr(74.8,57.7,1.3,0,42,T.Path));
  mesh(extrusion(lidFace,1.15,.24),shell,lid,0,0,-.91);
  slab(lid,75.1,58.1,.8,1.6,.2,bezel,0,42,-.45);
  // A machined lip around the glazing catches a thin highlight.
  const surround=rr(75.6,58.6,1.6,0,42);surround.holes.push(rr(74.3,57.3,1.1,0,42,T.Path));
  mesh(extrusion(surround,.34,.11),chrome,lid,0,0,-.23);
  const screenMat=track(new T.MeshBasicMaterial({color:0x080b10,toneMapped:false}));
  const screen=mesh(new T.PlaneGeometry(72.3,48.2),screenMat,lid,0,42,.02);
  const glass=material({color:0xbecedc,metalness:.12,roughness:.12,transparent:true,opacity:.055,clearcoat:1,clearcoatRoughness:.08,envMapIntensity:.6,depthWrite:false});
  mesh(new T.PlaneGeometry(73.5,55.2),glass,lid,0,42,.16);
  for(const x of [-35.4,35.4])for(const y of [7.3,74.1]) {
    disc(lid,1.75,.14,seam,x,y,.13);disc(lid,1.48,.25,rubber,x,y,.27);
  }
  // Rear lid badge, visible when folded.
  const back=new T.Group();back.position.set(0,40.9,-6.0);back.rotation.y=Math.PI;lid.add(back);
  slab(back,16.6,3.6,.24,1.8,.1,chrome,0,29,.12);
  slab(back,15.7,2.85,.1,1.4,.03,seam,0,29,.22);
  engraving('ANBERNIC',back,0,29,.3,14,2,'#c8cdd0');

  // Indicators wrap around the right edge, as in the video.
  const green=material({color:0xa1d67b,emissive:0x75bc45,emissiveIntensity:.6,roughness:.38});
  for(const [y,mat]of [[-18,green],[-21.5,dark]])slab(product,3.3,.8,.7,.2,.08,mat,39.2,y,.4);
  // Edge ports and switches have inset wells and separate metal/plastic faces.
  function sidePart(w,h,x,y,z,mat,rotation=Math.PI/2) {
    const p=slab(product,w,h,.3,.6,.08,mat,0,0,.15);p.position.set(x,y,z);p.rotation.y=rotation;return p;
  }
  sidePart(4.0,7,41,-32,-9,dark);sidePart(3.0,5.4,41.22,-32,-9,button);
  for(const y of [-54,-67])sidePart(1.1,10,41,y,-10,dark);
  sidePart(3.0,14,-41,-30,-9,dark,-Math.PI/2);
  sidePart(2.3,12.2,-41.25,-30,-9,button,-Math.PI/2);
  for(const [x,w]of [[-17,10],[0,8],[16,8]]) {
    const port=slab(product,w,3.8,.4,1,.1,dark,0,0,.2);port.rotation.x=-Math.PI/2;port.position.set(x,-1.8,-10.5);
  }
  // Lower shell screws become visible with a turn; no invented serial sticker.
  for(const x of [-32,32])for(const y of [-12,-76]) {
    const screw=new T.Group();screw.position.set(x,y,-16.3);screw.rotation.y=Math.PI;product.add(screw);
    disc(screw,1.65,.2,dark,0,0,0);disc(screw,1.1,.2,chrome,0,0,.15);engraving('+',screw,0,0,.27,1.5,1.5);
  }
  const shadowMap=textureCanvas(256,128,(ctx,w,h)=>{
    const g=ctx.createRadialGradient(w/2,h/2,0,w/2,h/2,w/2);g.addColorStop(0,'rgba(0,0,0,.3)');g.addColorStop(.5,'rgba(0,0,0,.14)');g.addColorStop(1,'rgba(0,0,0,0)');ctx.fillStyle=g;ctx.fillRect(0,0,w,h);
  });
  const shadow=mesh(new T.PlaneGeometry(125,25),track(new T.MeshBasicMaterial({map:shadowMap,transparent:true,depthWrite:false})),scene,4,-91,-25);

  let poster, posterRequest=0;
  const videoTexture=track(new T.VideoTexture(video));videoTexture.colorSpace=T.SRGBColorSpace;
  function loadPoster() {
    const id=++posterRequest;
    new T.TextureLoader().load(video.poster,texture=>{
      if(id!==posterRequest){texture.dispose();return;}
      poster?.dispose();poster=texture;texture.colorSpace=T.SRGBColorSpace;
      updateScreen();wake();
    },undefined,()=>{ /* The HTML fallback keeps its own poster. */ });
  }
  // Request texture upload explicitly: a visually hidden video may not receive
  // requestVideoFrameCallback even though its playback clock advances.
  function updateScreen() {
    if(video.readyState>=2)videoTexture.needsUpdate=true;
    const map=video.readyState>=2 ? videoTexture : poster;
    if(map && screenMat.map!==map){screenMat.map=map;screenMat.color.setHex(0xffffff);screenMat.needsUpdate=true;}
    if(map)stage.dataset.screenReady='true';
  }
  const observer=new MutationObserver(loadPoster);observer.observe(video,{attributes:true,attributeFilter:['poster']});

  const palettes={
    platinum:[0xaeb4b8,0x8d959a,0x686f71,.62],
    graphite:[0x353c44,0x242a31,0x3f454d,.38],
    'pearl blue':[0x9ebdcd,0x7699ae,0xa6b7c1,.4],
    'pearl pink':[0xd8acbb,0xb57f93,0xb8a9b0,.4],
    gold:[0xcfad67,0xa78039,0xbd9d5c,.65]
  };
  function finish(name) {
    const p=palettes[name]||palettes.platinum;shell.color.setHex(p[0]);edge.color.setHex(p[1]);button.color.setHex(p[2]);shell.metalness=p[3];rubber.color.setHex(p[1]);wake();
  }
  const onFinish=event=>finish(event.detail);
  stage.addEventListener('sp:finish',onFinish);

  let yaw=-.32,pitch=.08,roll=-.10,targetYaw=yaw,targetPitch=pitch;
  let closed=false,hinge=-.31,targetHinge=hinge;
  let pointer=null,lastFrame=0,lastRendered=0,frame=0,visible=true,disposed=false;
  let pressUntil=0,pressed=null;
  const hardware=[...stage.querySelectorAll('[data-hardware]')];
  const fallback=stage.querySelector('.device');
  const originals=hardware.map(element=>({element,parent:element.parentElement,next:element.nextSibling}));
  const fold=stage.closest('.sp-preview').querySelector('.model-fold');
  function foldLid() {
    closed=!closed;targetHinge=closed?Math.PI-.04:-.31;
    fold.setAttribute('aria-pressed',String(closed));fold.setAttribute('aria-label',closed?'Open lid':'Close lid');fold.title=closed?'Open lid':'Close lid';
    if(reduced.matches)hinge=targetHinge;
    stage.dataset.lid=closed?'closed':'open';stage.dispatchEvent(new CustomEvent('sp:lid'));wake();
  }
  fold.addEventListener('click',foldLid);
  const project=new T.Vector3();
  function placeButtons() {
    product.updateMatrixWorld(true);
    for(const element of hardware) {
      const anchor=controls.get(element.dataset.hardware);
      anchor.getWorldPosition(project);project.project(camera);
      element.style.left=`${(project.x*.5+.5)*100}%`;
      element.style.top=`${(-project.y*.5+.5)*100}%`;
      element.hidden=closed;
    }
  }
  function resize() {
    const {width,height}=stage.getBoundingClientRect();if(!width||!height)return;
    renderer.setSize(width,height,false);camera.aspect=width/height;
    camera.position.z=width<300?385:355;camera.updateProjectionMatrix();wake();
  }
  const resizeObserver=new ResizeObserver(resize);
  const visibilityObserver=new IntersectionObserver(entries=>{visible=entries[0].isIntersecting;if(visible)wake();else{cancelAnimationFrame(frame);frame=0;}},{threshold:.01});
  function wake() { if(!disposed && visible && !document.hidden && !frame)frame=requestAnimationFrame(draw); }
  function draw(now) {
    frame=0;if(disposed||!visible||document.hidden)return;
    if(now-lastRendered<1000/30){wake();return;}lastRendered=now;
    const dt=Math.min((now-lastFrame)/1000,.05)||.016;lastFrame=now;
    const ease=reduced.matches?1:1-Math.exp(-12*dt);
    yaw+=(targetYaw-yaw)*ease;pitch+=(targetPitch-pitch)*ease;hinge+=(targetHinge-hinge)*ease;
    product.rotation.set(pitch,yaw,roll);lid.rotation.x=hinge;
    product.position.y=27*(hinge+.31)/(Math.PI+.27);
    if(pressed && now>pressUntil){pressed.position.z+=.45;pressed=null;}
    updateScreen();placeButtons();
    renderer.shadowMap.needsUpdate=Math.abs(yaw-targetYaw)+Math.abs(pitch-targetPitch)+Math.abs(hinge-targetHinge)>.0001||!!pressed||!stage.dataset.modelReady;
    renderer.render(scene,camera);
    stage.dataset.modelReady='true';
    const moving=Math.abs(yaw-targetYaw)+Math.abs(pitch-targetPitch)+Math.abs(hinge-targetHinge)>.001;
    if(moving||pressed||(!video.paused&&video.readyState>=2))wake();
  }
  function down(event) {
    if(event.target.closest('button')||event.button!==0)return;
    pointer={id:event.pointerId,x:event.clientX,y:event.clientY,yaw:targetYaw,pitch:targetPitch};
    canvas.setPointerCapture(event.pointerId);stage.classList.add('is-dragging');
  }
  function move(event) {
    if(!pointer)return;
    const dx=event.clientX-pointer.x,dy=event.clientY-pointer.y;
    targetYaw=T.MathUtils.clamp(pointer.yaw+dx*.007,-1.2,1.2);
    targetPitch=T.MathUtils.clamp(pointer.pitch+dy*.004,-.35,.45);wake();
  }
  function up(event) {
    if(!pointer)return;
    if(canvas.hasPointerCapture(event.pointerId))canvas.releasePointerCapture(event.pointerId);
    pointer=null;stage.classList.remove('is-dragging');
  }
  canvas.addEventListener('pointerdown',down);canvas.addEventListener('pointermove',move);
  canvas.addEventListener('pointerup',up);canvas.addEventListener('pointercancel',up);
  function reset() { targetYaw=-.32;targetPitch=.08;wake(); }
  canvas.addEventListener('dblclick',reset);
  function keydown(event) {
    if(event.target!==stage)return;
    if(event.key==='ArrowLeft')targetYaw=Math.max(-1.2,targetYaw-.15);
    else if(event.key==='ArrowRight')targetYaw=Math.min(1.2,targetYaw+.15);
    else if(event.key==='ArrowUp')targetPitch=Math.max(-.35,targetPitch-.1);
    else if(event.key==='ArrowDown')targetPitch=Math.min(.45,targetPitch+.1);
    else if(event.key==='Home'){reset();event.preventDefault();return;}
    else return;
    event.preventDefault();wake();
  }
  stage.addEventListener('keydown',keydown);
  function press(event) {
    const element=event.target.closest('[data-hardware]');if(!element)return;
    const part=controls.get(element.dataset.hardware);
    if(pressed){pressed.position.z+=.45;pressed=null;}
    if(part?.isGroup){pressed=part;part.position.z-=.45;pressUntil=performance.now()+140;}
    wake();
  }
  stage.addEventListener('click',press);
  const videoEvents=['loadeddata','play','pause','seeked','timeupdate','emptied'];
  videoEvents.forEach(name=>video.addEventListener(name,wake));
  document.addEventListener('visibilitychange',wake);reduced.addEventListener('change',wake);
  function fallbackOnLoss(event) {event.preventDefault();cleanup();}
  canvas.addEventListener('webglcontextlost',fallbackOnLoss);
  function cleanup() {
    if(disposed)return;disposed=true;cancelAnimationFrame(frame);
    observer.disconnect();resizeObserver.disconnect();visibilityObserver.disconnect();
    videoEvents.forEach(name=>video.removeEventListener(name,wake));
    document.removeEventListener('visibilitychange',wake);reduced.removeEventListener('change',wake);
    stage.removeEventListener('sp:finish',onFinish);stage.removeEventListener('keydown',keydown);stage.removeEventListener('click',press);fold.removeEventListener('click',foldLid);
    for(const {element,parent,next}of originals){parent.insertBefore(element,next?.parentElement===parent?next:null);element.removeAttribute('style');element.hidden=false;}
    stage.classList.remove('model-ready');delete stage.dataset.modelReady;delete stage.dataset.screenReady;delete stage.dataset.lid;stage.dispatchEvent(new CustomEvent('sp:lid'));stage.removeAttribute('tabindex');stage.removeAttribute('aria-label');
    canvas.remove();poster?.dispose();for(const item of disposables)item.dispose();renderer.dispose();
  }
  // Commit only after the first complete render. Failure leaves the original
  // accessible HTML/video controls intact, including on devices without WebGL.
  try {
    product.rotation.set(pitch,yaw,roll);lid.rotation.x=hinge;
    const {width,height}=stage.getBoundingClientRect();renderer.setSize(width,height,false);camera.aspect=width/height;camera.updateProjectionMatrix();
    renderer.shadowMap.needsUpdate=true;renderer.render(scene,camera);
    stage.append(canvas,...hardware);stage.classList.add('model-ready');stage.dataset.lid='open';
    stage.tabIndex=0;stage.setAttribute('aria-label','3D RG SP. Drag or use arrow keys to turn. Home resets the view.');
    resizeObserver.observe(stage);visibilityObserver.observe(stage);loadPoster();resize();
    finish(document.querySelector('[data-shell][aria-pressed="true"]')?.dataset.shell || 'platinum');
  } catch(error) { cleanup();throw error; }
  return cleanup;
}
