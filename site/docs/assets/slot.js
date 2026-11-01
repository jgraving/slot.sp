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
      'pearl blue':['#dbe9f2','#a8c4d8','#6d8ba1','#d0dae3','#edf3f8','rgba(0,0,0,.44)'],
      'pearl pink':['#f6d3dd','#e6a6b8','#a76d7e','#dbd1d5','#f2eaed','rgba(0,0,0,.44)']
    };
    document.querySelectorAll('[data-shell]').forEach(button => {
      button.addEventListener('click', () => {
        document.querySelector('.device')?.classList.remove('secret-finish');
        const values = finishes[button.dataset.shell];
        ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,values[i]));
        document.querySelectorAll('[data-shell]').forEach(other => other.setAttribute('aria-pressed',String(other === button)));
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
    let visible = true;
    let finish = 'platinum';
    let request = 0;
    let position = 0;
    let segment = null;
    let segmentFrame = null;
    const media = new URL(preview.dataset.media, document.baseURI);
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
      if (wanted && visible && !document.hidden && stage?.dataset.lid !== 'closed') preview.play().catch(() => {});
      else preview.pause();
    }
    function state() {
      toggle.setAttribute('aria-pressed', String(wanted));
      toggle.querySelector('span').textContent = wanted ? 'Pause preview' : 'Play preview';
    }
    preview.addEventListener('play',state);
    preview.addEventListener('pause',state);
    toggle.addEventListener('click',() => { segment = null; wanted = !wanted; sync(); state(); });
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
      document.querySelector('.device').classList.add('secret-finish');
      const gold = ['#f2dfaa','#d3ad60','#92713c','#e5cf95','#fff1c9','rgba(57,39,18,.56)'];
      ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,gold[i]));
      document.querySelectorAll('[data-shell]').forEach(button => button.setAttribute('aria-pressed','false'));
      stage?.dispatchEvent(new CustomEvent('sp:finish',{detail:'gold'}));
      const notice = document.querySelector('.secret-notice');
      notice.replaceChildren(topicIcon('kid-mode', 'secret-icon'), document.createTextNode('You found the gold shell.'));
      clearTimeout(noticeTimer);
      noticeTimer = setTimeout(() => notice.replaceChildren(), 4000);
      playSegment(7.4, 9.25);
    });
    reduced.addEventListener('change',() => { if (reduced.matches) { wanted = false; sync(); state(); } });
    new IntersectionObserver(entries => { visible = entries[0].isIntersecting; sync(); },{threshold:.1}).observe(stage || preview);
    stage?.addEventListener('sp:lid',sync);
    // Load the self-hosted renderer only on the homepage, when it is in view.
    // The SVG and video remain usable if WebGL or the optional module fails.
    if (stage && !navigator.connection?.saveData) {
      const modelObserver = new IntersectionObserver(entries => {
        if (!entries.some(entry => entry.isIntersecting)) return;
        modelObserver.disconnect();
        import(new URL('sp-model.js',assets).href)
          .then(({mountSP}) => mountSP(stage,preview))
          .catch(error => console.warn('3D preview unavailable; using the illustrated preview.',error));
      },{rootMargin:'100px'});
      modelObserver.observe(stage);
    }
    document.addEventListener('visibilitychange',sync);
    state();
  }
  document.addEventListener('visibilitychange',() => {
    if (document.hidden) document.querySelectorAll('video').forEach(video => video.pause());
  });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded',init,{once:true});
  else init();
})();
