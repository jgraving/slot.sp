/* Reference-built RG SP. Coordinates are millimetres; +Z faces the player.
 * Anbernic's six-surface views, product photos and the supplied opening clips
 * guide the 82 × 83 × 23.9 mm closed envelope, hardware and painted finish.
 * This is a handcrafted reconstruction, not a photogrammetry scan. */
import * as T from './vendor/three-r180.min.js';

export function mountSP(stage, video, options={}) {
  if (!stage || stage.dataset.modelReady) return;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const renderer = new T.WebGLRenderer({alpha:true, antialias:true, powerPreference:'low-power'});
  renderer.setPixelRatio(options.capture ? 1.5 : Math.min(devicePixelRatio, 1.75));
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
  lid.position.set(0, 0, 2);
  product.add(lid);
  const controls = new Map();
  const etchMaterials = [];
  const disposables = new Set();
  const track = value => { disposables.add(value); return value; };
  const material = settings => track(new T.MeshPhysicalMaterial(settings));

  // Broad photographic softboxes, baked once into the reflection environment.
  const studio = new T.Scene();
  studio.background = new T.Color('#363c42');
  const room = new T.Mesh(new T.BoxGeometry(900,900,900),new T.MeshBasicMaterial({color:'#363c42',side:T.BackSide}));
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
  const fill = new T.PointLight(0xfffbf4,18000,0,2);fill.position.set(-110,160,170);scene.add(fill);
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
      const value=128+(seed>>>25); pixels.data.set([value,value,value,255],i);
    }
    ctx.putImageData(pixels,0,0);
  });
  grain.colorSpace=T.NoColorSpace; grain.wrapS=grain.wrapT=T.RepeatWrapping; grain.repeat.set(.025,.025);
  const flakes=textureCanvas(256,256,(ctx,w,h)=>{
    let seed=71;const pixels=ctx.createImageData(w,h);
    for(let i=0;i<pixels.data.length;i+=4){seed=(Math.imul(seed,1664525)+1013904223)>>>0;const v=231+(seed>>>28);pixels.data.set([v,v,v,255],i);}
    ctx.putImageData(pixels,0,0);
  });
  flakes.wrapS=flakes.wrapT=T.RepeatWrapping;flakes.repeat.set(.025,.025);
  // Pigmented metallic paint with fine flakes under a satin clear coat.
  // The shell is painted plastic, so avoid a mirror-chrome metal response.
  const paint={metalness:.72,roughness:.27,map:flakes,bumpMap:grain,bumpScale:.035,clearcoat:.55,clearcoatRoughness:.18,envMapIntensity:1.15};
  const shell=material({color:0xbfc3c4,...paint});
  const edge=material({color:0xaeb3b5,...paint});
  const button=material({color:0xb9bdc0,metalness:.2,roughness:.43,clearcoat:.18});
  const rubber=material({color:0x8b9295,metalness:0,roughness:.8});
  const dark=material({color:0x161b20,metalness:.12,roughness:.63});
  const seam=material({color:0x5d656d,metalness:.32,roughness:.6});
  const bezel=track(new T.MeshBasicMaterial({color:0x080b10,toneMapped:false}));
  const chrome=material({color:0xaab0b5,metalness:.8,roughness:.27});
  // Sparse metallic flakes have their own micro-facet normals. Their highlights
  // move with the shell, instead of baking bright dots into the color texture.
  for(const mat of [shell,edge]) {
    mat.onBeforeCompile=shader=>{
      shader.vertexShader=shader.vertexShader.replace('#include <common>','#include <common>\nvarying vec2 vPaintUV;')
        .replace('#include <begin_vertex>','#include <begin_vertex>\nvPaintUV = uv;');
      shader.fragmentShader=shader.fragmentShader.replace('#include <common>',`#include <common>
        varying vec2 vPaintUV;
        float flakeHash(vec2 p) { return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453); }
      `).replace('#include <lights_fragment_end>',`#include <lights_fragment_end>
        vec2 cell = floor(vPaintUV * 4.8);
        float seed = flakeHash(cell);
        vec2 dotUV = fract(vPaintUV * 4.8) - .5;
        float footprint = max(length(fwidth(vPaintUV * 4.8)), .01);
        float dotMask = (1.0 - smoothstep(.14, .24 + footprint, length(dotUV))) * min(1.0, .42 / footprint);
        vec3 baseN = normalize(vNormal);
        vec3 tangent = normalize(abs(baseN.z)<.95 ? cross(baseN,vec3(0,0,1)) : cross(baseN,vec3(0,1,0)));
        vec3 bitangent = cross(baseN,tangent);
        float azimuth = flakeHash(cell+19.7)*6.283185;
        float tilt = (flakeHash(cell+7.3)-.5)*1.15;
        vec3 flakeN = normalize(baseN + tilt*(cos(azimuth)*tangent+sin(azimuth)*bitangent));
        vec3 halfLight = normalize(normalize(vViewPosition)+normalize(vec3(-.45,.65,1.0)));
        float glint = pow(max(dot(flakeN,halfLight),0.0),72.0);
        float sparse = smoothstep(.90,.95,seed);
        reflectedLight.directSpecular += mix(vec3(.9),diffuseColor.rgb,.25)*glint*dotMask*sparse*2.2;
      `);
    };
    mat.customProgramCacheKey=()=> 'sp-metal-flake-v2';
  }

  function rr(w,h,r,cx=0,cy=0,Type=T.Shape) {
    const s=new Type(), x=cx-w/2,y=cy-h/2;
    s.moveTo(x+r,y);s.lineTo(x+w-r,y);s.quadraticCurveTo(x+w,y,x+w,y+r);
    s.lineTo(x+w,y+h-r);s.quadraticCurveTo(x+w,y+h,x+w-r,y+h);
    s.lineTo(x+r,y+h);s.quadraticCurveTo(x,y+h,x,y+h-r);
    s.lineTo(x,y+r);s.quadraticCurveTo(x,y,x+r,y); return s;
  }
  function circleHole(shape,x,y,r) { const p=new T.Path(); p.absarc(x,y,r,0,Math.PI*2,true);shape.holes.push(p); }
  function mesh(geometry,mat,parent,x=0,y=0,z=0) {
    if(mat===shell||mat===edge) {
      const p=geometry.attributes.position,n=geometry.attributes.normal,uv=geometry.attributes.uv;
      // Keep paint grain at a consistent physical scale on caps, edges and faces.
      for(let i=0;uv&&i<p.count;i++) {
        const nx=Math.abs(n.getX(i)),ny=Math.abs(n.getY(i)),nz=Math.abs(n.getZ(i));
        uv.setXY(i,nx>nz?p.getZ(i):p.getX(i),ny>Math.max(nx,nz)?p.getZ(i):p.getY(i));
      }
    }
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
    const ink=track(new T.MeshBasicMaterial({map,transparent:true,depthWrite:false,toneMapped:false}));
    if(color==='#42494d')etchMaterials.push(ink);
    return mesh(new T.PlaneGeometry(w,h),ink,parent,x,y,z);
  }

  // Two separately bevelled mouldings with a fine, continuous parting seam.
  slab(product,82,80,9.55,5.2,1.3,edge,0,-39,-6.15);
  slab(product,81.9,79.8,.24,5.0,.04,seam,0,-39,-5.95);
  const face=rr(80.6,78.3,4.0,0,-39);
  const wells=[[-21,-32.5,14.8],[21,-32.5,14.8],[0,-15.5,3.5],[-5.4,-72.5,3.6],[5.4,-72.5,3.6]];
  wells.forEach(([x,y,r])=>circleHole(face,x,y,r));
  // The perforated skin has a square bore; a bevel wider than these holes
  // would fold their walls across each other. Keep its outer moulding separate.
  // Leave clearance behind the thin face skin. A bevel on a same-size bore
  // otherwise intersects the sloping recess and creates a ragged metallic rim.
  const innerBody=face.clone();innerBody.holes=[];
  wells.forEach(([x,y,r])=>circleHole(innerBody,x,y,r+.9));
  mesh(extrusion(innerBody,5.85,.65),shell,product,0,0,-5.35);
  const speaker=[];
  for(let row=0;row<4;row++)for(let col=0;col<4;col++) {
    const x=(col-1.5)*4, y=-53.5+(row-1.5)*4, r=.31;
    speaker.push([x,y]);circleHole(face,x,y,r);
  }
  mesh(extrusion(face,.18,0),shell,product,0,0,-.18);
  for(const [x,y,r] of wells) {
    disc(product,r-.15,.25,shell,x,y,-1.4);
    ring(product,[[r,.02],[r-.25,-.2],[r-.85,-1.25],[r-.95,-1.4]],shell,x,y);
    ring(product,[[r-.95,-1.38],[r-.97,-1.45]],seam,x,y);
  }
  for(const [x,y] of speaker)disc(product,.305,.1,dark,x,y,-.13);

  function roundButton(x,y,r,label,action) {
    disc(product,r+.25,.18,dark,x,y,-1.2);
    const group=new T.Group(); group.position.set(x,y,-.85);product.add(group);
    ring(group,[[0,0],[r-.3,0],[r,.2],[r,.85],[r-.17,1.12],[r-.48,1.25],[r*.6,1.48],[0,1.55]],button,0,0);
    if(label)engraving(label,group,0,0,1.59,r*1.18,r*1.18);
    if(action)controls.set(action,group);
    return group;
  }
  roundButton(21,-24.3,3.85,'X');
  roundButton(12.8,-32.5,3.85,'Y');
  roundButton(29.2,-32.5,3.85,'A','insert');
  roundButton(21,-40.7,3.85,'B','eject');
  roundButton(0,-15.5,2.25,'M','boot');
  roundButton(-5.4,-72.5,2.6,''); roundButton(5.4,-72.5,2.6,'');
  engraving('SELECT',product,-5.4,-65.8,.1,8,2.1);
  engraving('START',product,5.4,-65.8,.1,7.1,2.1);
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
  disc(product,1.5,.04,button,-21,-32.5,1.25);
  for(const [action,x]of [['left',-28],['right',-14]]) {
    const anchor=new T.Object3D();anchor.position.set(x,-32.5,1.8);product.add(anchor);anchor.userData.pressPart=dpad;controls.set(action,anchor);
  }

  // Five knuckles follow the asymmetric joints in Anbernic's orthographic view.
  const joints=[-41,-34,-2,28,34,41];
  for(let i=0;i<joints.length-1;i++) {
    const length=joints[i+1]-joints[i]-.16,half=length/2,r=4.25;
    const profile=[[0,-half],[r-.45,-half],[r-.12,-half+.2],[r,-half+.5],[r,half-.5],[r-.12,half-.2],[r-.45,half],[0,half]];
    const g=new T.LatheGeometry(profile.map(([a,b])=>new T.Vector2(a,b)),64);g.rotateZ(-Math.PI/2);
    mesh(g,shell,product,(joints[i+1]+joints[i])/2,0,2);
  }
  for(const x of [-41,41]) {
    const cap=new T.Group();cap.position.set(x,0,2);cap.rotation.y=Math.sign(x)*Math.PI/2;product.add(cap);
    disc(cap,3.05,.12,edge,0,0,.08);disc(cap,2.6,.1,shell,0,0,.13);
  }
  slab(product,6.8,9,10,2.3,.6,shell,-37,-.5,2);
  slab(product,6.8,9,10,2.3,.6,shell,37,-.5,2);
  for(const [x,w]of [[-18,31],[31,5.7]])slab(lid,w,5.2,4.6,1.1,.3,shell,x,3,-.6);

  // The RG SP has a 3.4-inch 720 × 480 (3:2) panel. Native media fills it
  // exactly; the narrow black glass border is distinct from the painted bevel.
  slab(lid,82,73,5.1,4.0,.75,edge,0,40.5,-1.1);
  slab(lid,81.9,72.9,.20,4.0,.04,seam,0,40.5,-.99);
  const lidFace=rr(80.8,71.8,3.2,0,40.5);
  lidFace.holes.push(rr(75.4,52.2,1.2,0,42,T.Path));
  const lidSkin=mesh(extrusion(lidFace,1.15,.24),shell,lid,0,0,-.91);
  lidSkin.receiveShadow=false;
  slab(lid,75.6,52.4,.8,1.4,.2,bezel,0,42,-.18);
  const surround=rr(75.8,52.6,1.5,0,42);surround.holes.push(rr(74.5,51.1,.9,0,42,T.Path));
  mesh(extrusion(surround,.34,.11),shell,lid,0,0,-.23);
  const screenMat=track(new T.MeshBasicMaterial({color:0x080b10,toneMapped:false}));
  const screen=mesh(new T.PlaneGeometry(72.3,48.2),screenMat,lid,0,42,.02);
  const glass=material({color:0xbecedc,metalness:.12,roughness:.12,transparent:true,opacity:.085,clearcoat:1,clearcoatRoughness:.08,envMapIntensity:.6,depthWrite:false});
  mesh(new T.PlaneGeometry(73.5,49.4),glass,lid,0,42,.16);
  for(const x of [-35.4,35.4])for(const y of [8.2,72.5]) {
    disc(lid,1.85,.12,edge,x,y,.13);disc(lid,1.65,.25,rubber,x,y,.27);
  }
  // Rear lid badge, visible when folded.
  const back=new T.Group();back.position.set(0,40.5,-6.21);back.rotation.y=Math.PI;lid.add(back);
  slab(back,16.6,3.6,.24,1.8,.1,chrome,0,29,.12);
  slab(back,15.7,2.85,.1,1.4,.03,material({color:0x13161a,roughness:.22,metalness:.05,clearcoat:.8}),0,29,.22);
  // The rear is flipped around both axes when shut; keep the badge upright.
  engraving('ANBERNIC',back,0,29,.3,14,2,'#c8cdd0').rotation.z=Math.PI;

  // Each surface has its own hardware: TF1 + volume on the left; TF2,
  // power and reset on the right; two display/charging ports at the rear.
  function surface(x,y,z,rx=0,ry=0) {
    const group=new T.Group();group.position.set(x,y,z);group.rotation.set(rx,ry,0);product.add(group);return group;
  }
  function recess(parent,w,h,r,x,y) {
    slab(parent,w+.55,h+.55,.15,r+.2,.05,edge,x,y,.08);
    slab(parent,w,h,.12,r,.03,dark,x,y,.15);
  }
  const left=surface(-41.03,0,-8.2,0,-Math.PI/2);
  const right=surface(41.03,0,-8.2,0,Math.PI/2);
  recess(left,4.2,18,2.1,0,-24);
  slab(left,3.5,16.8,.7,1.7,.22,button,0,-24,.75);
  engraving('+',left,0,-19.2,.8,2.3,2.3);
  engraving('−',left,0,-28.8,.8,2.3,2.3);
  slab(left,3.2,.13,.1,.04,.02,seam,0,-24,.8);
  recess(right,4,7.6,2,0,-24);
  slab(right,3.25,6.9,.45,1.6,.15,button,0,-24,.52);
  const powerMap=textureCanvas(128,256,(ctx,w,h)=>{
    ctx.strokeStyle='#52585b';ctx.lineWidth=7;ctx.lineCap='round';ctx.beginPath();ctx.arc(w/2,h/2+7,34,-Math.PI*.34,Math.PI*1.34);ctx.stroke();ctx.beginPath();ctx.moveTo(w/2,h/2-45);ctx.lineTo(w/2,h/2+5);ctx.stroke();
  });
  mesh(new T.PlaneGeometry(2.8,5.5),track(new T.MeshBasicMaterial({map:powerMap,transparent:true,depthWrite:false})),right,0,-24,.57);
  disc(right,2.15,.1,edge,0,-35,.12);disc(right,1.65,.3,button,0,-35,.4);engraving('R',right,0,-35,.44,2,2);
  for(const [side,name]of [[left,'TF1/INT'],[right,'TF2/EXT']]) {
    recess(side,1.35,11.2,.35,0,-45);
    // Fingernail relief and the edge of the seated card.
    const notch=disc(side,1.8,.09,dark,0,-45,.18);notch.scale.x=.6;
    slab(side,.42,8.3,.12,.15,.03,seam,-.3,-45,.22);
    engraving(name,side,3,-45,.25,9.4,1.65).rotation.z=Math.PI/2;
  }
  const green=material({color:0xb8d7a1,emissive:0x83ad60,emissiveIntensity:.23,roughness:.4});
  const indicator=material({color:0xc9c3ad,roughness:.44});
  for(const [y,mat]of [[-14.7,green],[-19.2,indicator]]) {
    slab(product,2.8,.7,.4,.16,.05,mat,39.55,y,.2);
    slab(right,2.9,.7,.12,.16,.04,mat,-6.6,y,.1);
  }
  const rear=surface(0,1.03,-9.3,-Math.PI/2);
  // Rounded USB-C socket and the wider, bevelled mini-HDMI socket.
  for(const [x,w,label]of [[-7.5,8.3,'USB/OTG'],[7.5,10.5,'HD']]) {
    recess(rear,w,2.9,1.1,x,0);
    const rim=rr(w+.35,3.25,1.2,x,0);rim.holes.push(rr(w-.55,2.35,.8,x,0,T.Path));
    mesh(extrusion(rim,.22,.04),chrome,rear,0,0,.12);
    slab(rear,w-2.5,.5,.1,.16,.03,seam,x,.1,.21);
    engraving(label,rear,x,3.45,.22,label==='HD'?4:10,1.75);
  }
  // L1/R1 wrap around the corners. L2/R2 sit higher and slightly inset.
  for(const [x,label,high]of [[-36,'L1',false],[-24,'L2',true],[24,'R2',true],[36,'R1',false]]) {
    // Recessed mounting pockets and long skirts bury the caps in the shell.
    slab(rear,high?9.2:10.4,high?7.8:8.8,4.2,1.8,.4,shell,x,3,.9);
    slab(rear,high?7.8:8.7,high?6.3:7.2,2.6,1.2,.2,dark,x,3,1.02);
    slab(rear,high?7.2:8.1,high?5.8:6.6,high?5.4:4.5,1,.3,button,x,3,high?2.5:1.6);
    engraving(label,rear,x,3,high?2.55:1.65,3.7,2.8);
  }
  const front=surface(0,-79.03,-8.2,Math.PI/2);
  disc(front,2.65,.18,edge,0,0,.12);disc(front,2.05,.16,dark,0,0,.26);
  ring(front,[[2.08,.27],[1.82,.4],[1.55,.4],[1.55,.28]],chrome,0,0);
  // Paired openings around the lower left corner form the lanyard anchor.
  recess(left,2.5,3.2,.5,-1.8,-75.4);recess(front,2.5,3.2,.5,-36.8,-1.8);
  const underside=surface(0,0,-15.73,0,Math.PI);
  for(const x of [-36.4,36.4])for(const y of [-10.5,-69.5]) {
    disc(underside,2.2,.1,dark,x,y,.06);disc(underside,1.35,.14,chrome,x,y,.14);engraving('+',underside,x,y,.3,1.55,1.55);
  }
  engraving('ANBERNIC',underside,0,-29,.1,24,3.1);
  engraving('RG SP',underside,0,-33.8,.1,7,1.9);
  engraving('MADE IN CHINA',underside,0,-41,.1,18,1.4);
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
    platinum:[0xcbd0d2,0xb7bec2,0xb9bdc0,.74],
    graphite:[0x42474b,0x31363a,0x45494d,.63],
    'pearl blue':[0x9cd6d9,0x80b6bd,0xb9bec0,.69],
    'pearl pink':[0xe9bacd,0xcfa1b6,0xbebbc0,.67],
    gold:[0xcfad67,0xa78039,0xbd9d5c,.65]
  };
  function finish(name) {
    const p=palettes[name]||palettes.platinum;shell.color.setHex(p[0]);edge.color.setHex(p[1]);button.color.setHex(p[2]);shell.metalness=edge.metalness=p[3];rubber.color.setHex(p[1]);
    seam.color.setHex(p[1]).multiplyScalar(.5);
    const ink=name==='graphite'?2.8:1;etchMaterials.forEach(mat=>mat.color.setRGB(ink,ink,ink));
    stage.dataset.finish=name;wake();
  }
  const onFinish=event=>finish(event.detail);
  stage.addEventListener('sp:finish',onFinish);

  const defaultAngle=150,defaultYaw=-.38,defaultPitch=-.16;
  let opening=!reduced.matches&&!options.capture,introStart=null;
  let yaw=defaultYaw,pitch=defaultPitch,roll=-.08,targetYaw=yaw,targetPitch=pitch;
  // 180° is flat. The earlier negative rotation incorrectly went past flat.
  let angle=defaultAngle,closed=false,targetHinge=T.MathUtils.degToRad(180-angle),hinge=opening?Math.PI:targetHinge;
  let pointer=null,lastFrame=0,lastRendered=0,frame=0,visible=true,disposed=false;
  let pressUntil=0,pressed=null;
  const hardware=[...stage.querySelectorAll('[data-hardware]')];
  const fallback=stage.querySelector('.sp-render-fallback');
  const videoParent=video.parentElement,videoNext=video.nextSibling;
  const originals=hardware.map(element=>({element,parent:element.parentElement,next:element.nextSibling}));
  const fold=stage.closest('.sp-preview').querySelector('.model-fold');
  const lidInput=stage.closest('.sp-preview').querySelector('.model-angle input');
  const lidOutput=stage.closest('.sp-preview').querySelector('.model-angle output');
  function setLid(value,manual=true) {
    if(manual&&opening){opening=false;stage.dataset.opening='false';}
    angle=T.MathUtils.clamp(Number(value),0,180);closed=angle<12;
    targetHinge=T.MathUtils.degToRad(180-angle);
    hardware.forEach(element=>{element.hidden=angle<45;});
    lidInput.value=String(angle);lidOutput.value=`${angle}°`;
    lidInput.setAttribute('aria-valuetext',angle===180?'180 degrees, fully flat':angle===0?'Closed':`${angle} degrees`);
    fold.setAttribute('aria-pressed',String(closed));fold.setAttribute('aria-label',closed?'Open lid':'Close lid');fold.title=closed?'Open lid':'Close lid';
    stage.dataset.lidAngle=String(angle);
    stage.dataset.lid=closed?'closed':'open';stage.dispatchEvent(new CustomEvent('sp:lid'));wake();
  }
  function foldLid(){setLid(closed?defaultAngle:0);}
  function adjustLid(){setLid(lidInput.value);}
  fold.addEventListener('click',foldLid);
  lidInput.addEventListener('input',adjustLid);
  const project=new T.Vector3();
  const normal=new T.Vector3(),toCamera=new T.Vector3();
  function placeButtons() {
    product.updateMatrixWorld(true);
    for(const element of hardware) {
      const anchor=controls.get(element.dataset.hardware);
      anchor.getWorldPosition(project);
      normal.set(0,0,1).applyQuaternion(product.quaternion);toCamera.copy(camera.position).sub(project).normalize();
      element.hidden=opening||(180-T.MathUtils.radToDeg(hinge))<45||normal.dot(toCamera)<.2;
      project.project(camera);
      element.style.left=`${(project.x*.5+.5)*100}%`;
      element.style.top=`${(-project.y*.5+.5)*100}%`;
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
    const poseChanged=Math.abs(yaw-targetYaw)+Math.abs(pitch-targetPitch)+Math.abs(hinge-targetHinge)>.0001;
    yaw+=(targetYaw-yaw)*ease;pitch+=(targetPitch-pitch)*ease;
    if(opening) {
      introStart??=now;const t=T.MathUtils.clamp((now-introStart-80)/900,0,1);
      hinge=T.MathUtils.lerp(Math.PI,targetHinge,1-Math.pow(1-t,3));
      if(t===1){opening=false;stage.dataset.opening='false';stage.dispatchEvent(new CustomEvent('sp:lid'));}
    } else hinge+=(targetHinge-hinge)*ease;
    product.rotation.set(pitch,yaw,roll);lid.rotation.x=hinge;
    product.position.y=(79-Math.max(4,77*Math.cos(hinge)))/2-2;
    if(pressed && now>pressUntil){pressed.position.z+=.45;pressed=null;}
    updateScreen();placeButtons();
    renderer.shadowMap.needsUpdate=opening||poseChanged||!!pressed||!stage.dataset.modelReady;
    renderer.render(scene,camera);
    stage.dataset.modelReady='true';
    stage.dataset.pose=opening?'opening':Math.abs(hinge-targetHinge)<.002 ? (closed?'closed':angle===180?'flat':'open') : 'moving';
    stage.dataset.renderedAngle=String(180-T.MathUtils.radToDeg(hinge));
    stage.dataset.viewYaw=String(yaw);stage.dataset.viewPitch=String(pitch);
    const moving=Math.abs(yaw-targetYaw)+Math.abs(pitch-targetPitch)+Math.abs(hinge-targetHinge)>.001;
    if(opening||moving||pressed||(!video.paused&&video.readyState>=2))wake();
  }
  function down(event) {
    if(event.target.closest('button')||event.button!==0)return;
    pointer={id:event.pointerId,x:event.clientX,y:event.clientY,yaw:targetYaw,pitch:targetPitch};
    canvas.setPointerCapture(event.pointerId);stage.classList.add('is-dragging');
  }
  function move(event) {
    if(!pointer)return;
    const dx=event.clientX-pointer.x,dy=event.clientY-pointer.y;
    targetYaw=pointer.yaw+dx*.007;
    targetPitch=T.MathUtils.clamp(pointer.pitch+dy*.004,-1.1,1.1);wake();
  }
  function up(event) {
    if(!pointer)return;
    if(canvas.hasPointerCapture(event.pointerId))canvas.releasePointerCapture(event.pointerId);
    pointer=null;stage.classList.remove('is-dragging');
  }
  canvas.addEventListener('pointerdown',down);canvas.addEventListener('pointermove',move);
  canvas.addEventListener('pointerup',up);canvas.addEventListener('pointercancel',up);
  function reset() { targetYaw=defaultYaw;targetPitch=defaultPitch;wake(); }
  canvas.addEventListener('dblclick',reset);
  function keydown(event) {
    if(event.target!==stage)return;
    if(event.key==='ArrowLeft')targetYaw-=.15;
    else if(event.key==='ArrowRight')targetYaw+=.15;
    else if(event.key==='ArrowUp')targetPitch=Math.max(-1.1,targetPitch-.1);
    else if(event.key==='ArrowDown')targetPitch=Math.min(1.1,targetPitch+.1);
    else if(event.key==='Home'){reset();event.preventDefault();return;}
    else return;
    event.preventDefault();wake();
  }
  stage.addEventListener('keydown',keydown);
  function press(event) {
    const element=event.target.closest('[data-hardware]');if(!element)return;
    const anchor=controls.get(element.dataset.hardware);
    const part=anchor?.userData.pressPart || anchor;
    if(pressed){pressed.position.z+=.45;pressed=null;}
    if(part?.isGroup || part?.isMesh){pressed=part;part.position.z-=.45;pressUntil=performance.now()+140;}
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
    stage.removeEventListener('sp:finish',onFinish);stage.removeEventListener('keydown',keydown);stage.removeEventListener('click',press);fold.removeEventListener('click',foldLid);lidInput.removeEventListener('input',adjustLid);
    for(const {element,parent,next}of originals){parent.insertBefore(element,next?.parentElement===parent?next:null);element.removeAttribute('style');element.hidden=false;}
    videoParent.insertBefore(video,videoNext?.parentElement===videoParent?videoNext:null);
    stage.classList.remove('model-ready');delete stage.dataset.modelReady;delete stage.dataset.screenReady;delete stage.dataset.lid;stage.removeAttribute('tabindex');stage.removeAttribute('role');stage.removeAttribute('aria-label');delete stage.dataset.pose;
    for(const key of ['lidAngle','viewYaw','viewPitch','finish'])delete stage.dataset[key];
    canvas.remove();poster?.dispose();for(const item of disposables)item.dispose();renderer.dispose();
    stage.dispatchEvent(new CustomEvent('sp:fallback'));
  }
  // Commit only after the first complete render. Failure leaves the original
  // accessible HTML/video controls intact, including on devices without WebGL.
  try {
    product.rotation.set(pitch,yaw,roll);lid.rotation.x=hinge;
    product.position.y=(79-Math.max(4,77*Math.cos(hinge)))/2-2;
    const {width,height}=stage.getBoundingClientRect();renderer.setSize(width,height,false);camera.aspect=width/height;camera.updateProjectionMatrix();
    renderer.shadowMap.needsUpdate=true;renderer.render(scene,camera);
    stage.append(canvas,video,...hardware);stage.classList.add('model-ready');stage.dataset.opening=String(opening);setLid(defaultAngle,false);
    stage.tabIndex=0;stage.setAttribute('role','group');stage.setAttribute('aria-label','3D RG SP. Drag or use arrow keys to turn. Home resets the view.');
    resizeObserver.observe(stage);visibilityObserver.observe(stage);if(!options.capture)loadPoster();resize();
    finish(fallback?.classList.contains('secret-finish') ? 'gold' : (document.querySelector('[data-shell][aria-pressed="true"]')?.dataset.shell || 'platinum'));
  } catch(error) { cleanup();throw error; }
  if(options.capture) {
    visible=false;cancelAnimationFrame(frame);frame=0;observer.disconnect();resizeObserver.disconnect();visibilityObserver.disconnect();
    let capturedAngle=null;
    cleanup.captureFrame=async({lidAngle=150,color='platinum',source=null}={})=>{
      finish(color);setLid(lidAngle);hinge=targetHinge;
      if(source){const texture=await new T.TextureLoader().loadAsync(source);texture.colorSpace=T.SRGBColorSpace;poster?.dispose();poster=texture;}
      screenMat.map=source?poster:null;screenMat.color.setHex(source?0xffffff:0x080b10);screenMat.needsUpdate=true;
      product.rotation.set(defaultPitch,defaultYaw,roll);lid.rotation.x=hinge;product.position.y=(79-Math.max(4,77*Math.cos(hinge)))/2-2;
      product.updateMatrixWorld(true);renderer.shadowMap.needsUpdate=capturedAngle!==lidAngle;renderer.render(scene,camera);capturedAngle=lidAngle;
      const {width,height}=stage.getBoundingClientRect();
      const point=object=>{object.getWorldPosition(project);project.project(camera);return {x:(project.x*.5+.5)*width,y:(-.5*project.y+.5)*height};};
      const screenQuad=[[-36.15,24.1],[36.15,24.1],[36.15,-24.1],[-36.15,-24.1]].map(([x,y])=>{
        project.set(x,y,0);screen.localToWorld(project);project.project(camera);return {x:(project.x*.5+.5)*width,y:(-.5*project.y+.5)*height};
      });
      return {screenQuad,controls:Object.fromEntries([...controls].map(([name,anchor])=>[name,point(anchor)]))};
    };
  }
  return cleanup;
}
