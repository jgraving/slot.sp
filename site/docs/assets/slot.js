(() => {
  const assets = new URL('../', document.currentScript.src);
  const topics = {
    guide:'guide', 'getting-started':'getting-started', controls:'controls',
    library:'library', playing:'playing', 'kid-mode':'kid-mode', bluetooth:'bluetooth',
    display:'display', 'night-display':'night-display', screenshots:'screenshots',
    'power-options':'power-options', 'link-play':'link-play', 'cart-studio':'cart-studio',
    settings:'settings', storage:'storage', customizing:'cart-studio', troubleshooting:'help'
  };
  function topicIcon(name, className) {
    const icon = document.createElement('img');
    icon.src = new URL(`icons/${name}.svg`, assets).href;
    icon.alt = '';
    icon.setAttribute('aria-hidden', 'true');
    icon.width = icon.height = 24;
    icon.className = className;
    return icon;
  }
  function init() {
    document.querySelectorAll('.md-sidebar--primary a.md-nav__link').forEach(link => {
      const topic = new URL(link.href).pathname.split('/').filter(Boolean).at(-1);
      if (topics[topic]) {
        link.classList.add('nav-topic');
        link.prepend(topicIcon(topics[topic], 'nav-topic-icon'));
      }
    });
    const topic = location.pathname.split('/').filter(Boolean).at(-1);
    const heading = document.querySelector('article h1');
    if (topics[topic] && heading) heading.prepend(topicIcon(topics[topic], 'page-topic-icon'));

    const finishes = {
      graphite:['#5a5d64','#34363b','#16171a','#3c3e43','#5c5f66','rgba(255,255,255,.42)'],
      platinum:['#e2e5e9','#b8bcc2','#7e838b','#c2c6cc','#e9ecf0','rgba(0,0,0,.46)'],
      'pearl blue':['#d9eceb','#91c7cb','#628f96','#d0d8da','#edf3f8','rgba(0,0,0,.44)'],
      'pearl pink':['#f6d3dd','#e6a6b8','#a76d7e','#dbd1d5','#f2eaed','rgba(0,0,0,.44)']
    };
    document.querySelectorAll('[data-shell]').forEach(button => {
      button.addEventListener('click', () => {
        document.querySelector('.sp-render-fallback')?.classList.remove('secret-finish');
        const values = finishes[button.dataset.shell];
        ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,values[i]));
        document.querySelectorAll('[data-shell]').forEach(other => other.setAttribute('aria-pressed',String(other === button)));
        document.querySelector('.finish-name').textContent = button.getAttribute('aria-label');
        try { localStorage.setItem('slot.sp:hardware-finish',button.dataset.shell); } catch { /* Color switching also works without storage. */ }
        changePreview(button);
        document.querySelector('.sp-stage')?.dispatchEvent(new CustomEvent('sp:finish',{detail:button.dataset.shell}));
      });
    });

    const preview = document.getElementById('hero-preview');
    const stage = document.querySelector('.sp-stage');
    const toggle = document.querySelector('.preview-toggle');
    if (!preview || !toggle) return;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    let wanted = !reduced.matches && !navigator.connection?.saveData;
    stage.dataset.opening=String(wanted);
    let visible = false;
    let finish = 'platinum';
    let request = 0;
    let position = 0;
    let segment = null;
    let segmentFrame = null;
    const media = new URL(preview.dataset.media, document.baseURI);
    const modelMedia = new URL(stage.dataset.renderMedia, document.baseURI);
    const still = stage.querySelector('.model-poster img');
    const picture = still.parentElement;
    const motion = stage.querySelector('.model-opening');
    const fold = document.querySelector('.model-fold');
    const canAnimate = () => !reduced.matches && !navigator.connection?.saveData;
    let shell = 'platinum', ready = false, introPending = canAnimate();
    let pose = introPending ? 'closed' : 'open', motionTarget = null, motionRequest = 0, posterRequest = 0;
    const busy = () => pose === 'opening' || pose === 'closing';
    const colorToken = () => shell.replaceAll(' ', '_');
    const asset = name => new URL(name, modelMedia).href;
    async function updateStill(open, fade = false) {
      const current = ++posterRequest, previous = still.currentSrc;
      const url = asset(`${colorToken()}-${open ? 'open' : 'closed'}.png`);
      still.src = url;
      picture.querySelector('source').srcset = url;
      still.alt = `${shell} RG SP with metallic paint`;
      try { await still.decode(); } catch { /* Keep the previous painted image on a failed request. */ }
      if (current !== posterRequest) return;
      if (fade && canAnimate() && previous && previous !== url) {
        picture.style.backgroundImage = `url(${JSON.stringify(previous)})`;
        const change = still.animate([{opacity:0},{opacity:1}], {duration:200,easing:'cubic-bezier(.2,0,0,1)'});
        change.finished.then(() => { if (current === posterRequest) picture.style.backgroundImage = ''; }).catch(() => {});
      }
    }
    function setPose(next) {
      pose = next;
      stage.dataset.pose = next;
      stage.dataset.opening = String(busy());
      stage.dataset.lid = next === 'closed' || next === 'closing' ? 'closed' : 'open';
      stage.dataset.lidAngle = stage.dataset.lid === 'closed' ? '0' : '150';
      fold.disabled = !ready || busy();
      const closed = next === 'closed';
      fold.setAttribute('aria-label', closed ? 'Open lid' : 'Close lid');
      fold.setAttribute('aria-expanded', String(!closed));
      fold.setAttribute('aria-pressed', String(closed));
      fold.title = closed ? 'Open lid' : 'Close lid';
      stage.querySelectorAll('[data-hardware]').forEach(button => { button.hidden = !ready || next !== 'open'; });
      sync();
      state();
    }
    async function finishMotion(current = motionRequest) {
      if (current !== motionRequest || !motionTarget) return;
      motion.pause();
      const target = motionTarget;
      let color;
      do { color = shell; await updateStill(target === 'open'); }
      while (color !== shell && current === motionRequest);
      if (current !== motionRequest) return;
      motionTarget = null;
      setPose(target);
    }
    async function flipLid(open, animate = true) {
      introPending = false;
      const current = ++motionRequest;
      motion.pause();
      motionTarget = open ? 'open' : 'closed';
      if (!animate || !canAnimate()) { await finishMotion(current); return; }
      motion.poster = asset(`${colorToken()}-${open ? 'closed' : 'open'}.png`);
      motion.src = asset(`${colorToken()}-${open ? 'opening' : 'closing'}.mp4`);
      motion.load();
      setPose(open ? 'opening' : 'closing');
      if (visible && !document.hidden) motion.play().catch(() => finishMotion(current));
    }
    motion.addEventListener('ended', () => finishMotion());
    motion.addEventListener('error', () => finishMotion());
    fold.addEventListener('click', () => flipLid(pose === 'closed'));
    stage.addEventListener('sp:finish', event => {
      shell = event.detail;
      stage.dataset.finish = shell;
      if (!busy()) updateStill(pose === 'open', true);
    });
    async function prepareModel() {
      try {
        const response = await fetch(asset('projection.json'));
        if (!response.ok) throw Error('Model projection unavailable');
        const projection = await response.json();
        const [p0,p1,p2,p3] = projection.screenQuad;
        const dx1=p1.x-p2.x,dx2=p3.x-p2.x,dx3=p0.x-p1.x+p2.x-p3.x;
        const dy1=p1.y-p2.y,dy2=p3.y-p2.y,dy3=p0.y-p1.y+p2.y-p3.y,den=dx1*dy2-dx2*dy1;
        const g=(dx3*dy2-dx2*dy3)/den,h=(dx1*dy3-dx3*dy1)/den;
        const matrix=[(p1.x-p0.x+g*p1.x)/720,(p1.y-p0.y+g*p1.y)/720,0,g/720,(p3.x-p0.x+h*p3.x)/480,(p3.y-p0.y+h*p3.y)/480,0,h/480,0,0,1,0,p0.x,p0.y,0,1];
        stage.style.setProperty('--fallback-screen', `matrix3d(${matrix.join(',')})`);
        stage.querySelectorAll('[data-hardware]').forEach(button => {
          const point = projection.controls[button.dataset.hardware];
          button.style.left = point.x + 'px'; button.style.top = point.y + 'px';
        });
        ready = true;
        stage.dataset.ready = 'true';
        stage.dataset.finish = shell;
        setPose(pose);
        if (introPending && visible) flipLid(true); else sync();
      } catch (error) {
        introPending = false;
        updateStill(true);
        setPose('open');
        console.warn('Handheld controls unavailable.', error);
      }
    }
    updateStill(pose === 'open');
    setPose(pose);
    function changePreview(button) {
      const next = button.dataset.shell.replaceAll(' ', '_');
      if (next === finish) return;
      segment = null;
      finish = next;
      if (preview.readyState > 0) position = preview.currentTime;
      const current = ++request;
      preview.pause();
      preview.poster = new URL(`hero-dark-${finish}.png`, media).href;
      preview.setAttribute('aria-label', `Watch a cartridge enter the slot, eject and return to the carousel, ${button.getAttribute('aria-label')} finish`);
      preview.addEventListener('loadedmetadata', () => {
        if (current !== request) return;
        preview.currentTime = Math.min(position, Math.max(0, preview.duration - .01));
        sync();
      }, {once:true});
      preview.src = new URL(`hero-dark-${finish}.mp4`, media).href;
      preview.load();
      state();
    }
    function sync() {
      if (busy()) {
        const current = motionRequest;
        if (visible && !document.hidden) motion.play().catch(() => finishMotion(current)); else motion.pause();
      }
      if (wanted && visible && !document.hidden && pose === 'open' && ready) preview.play().catch(() => {});
      else preview.pause();
    }
    function state() {
      const playing = wanted && pose === 'open';
      toggle.setAttribute('aria-pressed', String(playing));
      toggle.querySelector('span').textContent = playing ? 'Pause preview' : 'Play preview';
    }
    preview.addEventListener('play',state);
    preview.addEventListener('pause',state);
    toggle.addEventListener('click',() => { segment = null; wanted = pose === 'open' ? !wanted : true; if (pose === 'closed' && ready) flipLid(true); sync(); state(); });
    // Use the existing native clip, including the slot's real occlusion.
    // These controls scrub a preview; no game or emulation runs in the page.
    function playSegment(start, end) {
      const current = ++request;
      function begin() {
        if (current !== request) return;
        if (segmentFrame !== null) preview.cancelVideoFrameCallback(segmentFrame);
        segmentFrame = null;
        preview.currentTime = Math.min(start, preview.duration - .01);
        segment = end;
        wanted = end > start;
        sync();
        state();
        if (wanted) watchSegment();
      }
      if (preview.readyState > 0) begin();
      else {
        preview.addEventListener('loadedmetadata', begin, {once:true});
        preview.load();
      }
    }
    function stopAtSegment(time) {
      if (segment === null || time < segment) return false;
      const end = segment;
      segment = null;
      preview.pause();
      preview.currentTime = end;
      wanted = false;
      state();
      return true;
    }
    function watchSegment() {
      if (segmentFrame !== null || segment === null || !preview.requestVideoFrameCallback) return;
      segmentFrame = preview.requestVideoFrameCallback((now, frame) => {
        segmentFrame = null;
        if (!stopAtSegment(frame.mediaTime) && !preview.paused) watchSegment();
      });
    }
    preview.addEventListener('playing', watchSegment);
    preview.addEventListener('timeupdate', () => {
      // Hidden DOM video can stop delivering frame callbacks while its texture plays.
      stopAtSegment(preview.currentTime);
    });
    document.querySelectorAll('[data-hardware]').forEach(button => {
      button.addEventListener('click', () => {
        switch (button.dataset.hardware) {
          case 'insert': playSegment(.45, 1.10); break;
          case 'eject': playSegment(1.12, 3.0); break;
          case 'boot': playSegment(7.4, 9.25); break;
          case 'left': playSegment(5.15, 7.38); break;
          case 'right': playSegment(3.02, 5.13); break;
        }
      });
    });
    // A small, opt-in old-console secret. Normal typing and shortcuts are untouched.
    const code = ['ArrowUp','ArrowUp','ArrowDown','ArrowDown','ArrowLeft','ArrowRight','ArrowLeft','ArrowRight','b','a'];
    let codeIndex = 0;
    let lastKey = 0;
    let noticeTimer;
    document.addEventListener('keydown', event => {
      if (event.repeat || event.altKey || event.ctrlKey || event.metaKey ||
          event.target.closest('input,textarea,select,[contenteditable="true"]')) return;
      if (performance.now() - lastKey > 3000) codeIndex = 0;
      lastKey = performance.now();
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      codeIndex = key === code[codeIndex] ? codeIndex + 1 : (key === code[0] ? 1 : 0);
      if (codeIndex !== code.length) return;
      codeIndex = 0;
      document.querySelector('.sp-render-fallback').classList.add('secret-finish');
      const gold = ['#f2dfaa','#d3ad60','#92713c','#e5cf95','#fff1c9','rgba(57,39,18,.56)'];
      ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,gold[i]));
      document.querySelectorAll('[data-shell]').forEach(button => button.setAttribute('aria-pressed','false'));
      document.querySelector('.finish-name').textContent = 'Gold';
      stage?.dispatchEvent(new CustomEvent('sp:finish',{detail:'gold'}));
      const notice = document.querySelector('.secret-notice');
      notice.replaceChildren(topicIcon('kid-mode', 'secret-icon'), document.createTextNode('You found the gold shell.'));
      clearTimeout(noticeTimer);
      noticeTimer = setTimeout(() => notice.replaceChildren(), 4000);
      playSegment(7.4, 9.25);
    });
    reduced.addEventListener('change', () => {
      if (reduced.matches) {
        wanted = false; introPending = false;
        if (busy()) finishMotion();
        sync(); state();
      }
    });
    new IntersectionObserver(entries => {
      visible = entries[0].isIntersecting;
      if (visible && ready && introPending) flipLid(true); else sync();
    }, {threshold:.08}).observe(stage);
    prepareModel();
    document.addEventListener('visibilitychange',sync);
    state();
    try {
      const saved = localStorage.getItem('slot.sp:hardware-finish');
      if (saved && finishes[saved]) [...document.querySelectorAll('[data-shell]')].find(button => button.dataset.shell === saved)?.click();
    } catch { /* Storage may be disabled. */ }
  }
  document.addEventListener('visibilitychange',() => {
    if (document.hidden) document.querySelectorAll('video').forEach(video => video.pause());
  });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded',init,{once:true});
  else init();
})();
