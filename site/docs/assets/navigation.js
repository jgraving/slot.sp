/* Website and bundled Studio share a velocity-preserving navigation spring. */
(() => {
  function init(root, selector, key) {
    if (!root || root.classList.contains('slot-selector')) return;
    const links = [...root.querySelectorAll(selector)];
    if (!links.length) return;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    const surface = document.createElement('span');
    surface.className = 'selector-surface';
    surface.setAttribute('aria-hidden', 'true');
    root.classList.add('slot-selector');
    links.forEach(link => link.classList.add('selector-link'));
    root.prepend(surface);
    const names = ['x','y','width','height'];
    let hovered = null, frame = 0, last = 0, current = null, target = null, bridging = false;
    const headerBridge = !key && !!root.closest('header');
    const velocity = Object.fromEntries(names.map(name => [name,0]));
    const visible = link => link && link.getBoundingClientRect().height > 0;
    function paint() {
      for (const name of names) surface.style.setProperty(`--selector-${name}`, `${current[name]}px`);
    }
    function stop() { cancelAnimationFrame(frame); frame = 0; last = 0; }
    function snap(next) {
      stop(); current = {...next};
      names.forEach(name => velocity[name] = 0);
      paint();
    }
    function step(now) {
      // M3 Expressive fast spatial: stiffness 800, damping ratio .6, mass 1.
      // Integrate analytically; retargeting preserves current position/velocity.
      const dt = Math.min((now - (last || now - 16.667)) / 1000, .064);
      last = now;
      const omega = Math.sqrt(800), damping = .6 * omega;
      const frequency = omega * Math.sqrt(1 - .6 ** 2);
      const decay = Math.exp(-damping * dt);
      const cosine = Math.cos(frequency * dt), sine = Math.sin(frequency * dt);
      let settled = true;
      for (const name of names) {
        const offset = current[name] - target[name], speed = velocity[name];
        const coefficient = (speed + damping * offset) / frequency;
        const wave = offset * cosine + coefficient * sine;
        current[name] = target[name] + decay * wave;
        velocity[name] = decay * (-damping * wave + frequency * (-offset * sine + coefficient * cosine));
        if (Math.abs(current[name] - target[name]) > .05 || Math.abs(velocity[name]) > .5) settled = false;
      }
      paint();
      if (settled) snap(target);
      else frame = requestAnimationFrame(step);
    }
    function destination() {
      const focus = links.find(link => link.contains(document.activeElement) && link.matches(':focus-visible'));
      const active = links.find(link => visible(link) && (link.matches('[aria-current],.md-nav__link--active') || link.closest('.md-tabs__item--active')));
      return [hovered, focus, active].find(visible);
    }
    function place(immediate = false) {
      const link = destination();
      surface.style.opacity = link ? '1' : '0';
      if (!link) { stop(); return; }
      const box = root.getBoundingClientRect(), rect = link.getBoundingClientRect();
      target = {x:rect.left-box.left+root.scrollLeft,y:rect.top-box.top+root.scrollTop,width:rect.width,height:rect.height};
      if (!current || immediate || reduced.matches) snap(target);
      else if (!frame) frame = requestAnimationFrame(step);
    }
    function remember() {
      if (headerBridge && document.documentElement.dataset.pageMotion === 'fallback') {
        try { sessionStorage.setItem('slot-header-bridge-v2', JSON.stringify({
          url:location.href,at:Date.now(),viewport:innerWidth,rootWidth:root.clientWidth,rect:current,opacity:Number(surface.style.opacity)
        })); } catch {}
      }
      if (!key || !current || surface.style.opacity !== '1') return;
      try { sessionStorage.setItem(key, JSON.stringify(current)); } catch {}
    }
    function restoreBridge() {
      if (!headerBridge || reduced.matches || document.documentElement.dataset.pageFallback !== 'incoming') return;
      try {
        const previous = JSON.parse(sessionStorage.getItem('slot-header-bridge-v2'));
        if (!previous || previous.url === location.href || Date.now()-previous.at > 30000 ||
            Math.abs(previous.viewport-innerWidth)>1 || Math.abs(previous.rootWidth-root.clientWidth)>1) return;
        if (previous.rect && names.every(name => Number.isFinite(previous.rect[name])) &&
            previous.rect.width>0 && previous.rect.height>0) snap(previous.rect);
        surface.style.opacity=String(previous.opacity);
        bridging=true;
      } catch {}
    }
    place(true);
    restoreBridge();
    if (!reduced.matches && key) {
      try {
        const previous = JSON.parse(sessionStorage.getItem(key));
        if (previous && names.every(name => Number.isFinite(previous[name])) && previous.width > 0 && previous.height > 0 && previous.x >= 0 && previous.y >= 0 && previous.x+previous.width <= root.clientWidth+2 && previous.y+previous.height <= root.scrollHeight+2) snap(previous);
      } catch {}
    }
    root.setAttribute('data-selector-ready', '');
    if (key || bridging) requestAnimationFrame(() => place());
    root.addEventListener('pointerover', event => { hovered = links.find(link => link.contains(event.target)) || null; place(); });
    root.addEventListener('pointerleave', () => { hovered = null; place(); });
    root.addEventListener('focusin', () => place());
    root.addEventListener('focusout', () => requestAnimationFrame(() => place()));
    root.addEventListener('click', remember);
    addEventListener('pagehide', () => { remember(); stop(); });
    addEventListener('pageswap', stop);
    document.addEventListener('slot:page-leaving', () => { remember(); stop(); });
    document.addEventListener('slot:page-ready', () => { bridging=false; });
    // The native shared element takes over from the outgoing spring's exact
    // visible position. The incoming capture must already be at its destination.
    addEventListener('pagereveal', () => {
      if (document.documentElement.dataset.pageMotion === 'native') { hovered = null; place(true); }
    });
    addEventListener('pageshow', () => { hovered = null; restoreBridge(); place(!bridging); });
    if (typeof ResizeObserver !== 'undefined') new ResizeObserver(() => place(!bridging)).observe(root);
    else addEventListener('resize', () => place(true));
    document.fonts?.ready.then(() => place(!bridging));
    reduced.addEventListener('change', () => place(true));
  }
  function start() {
    const header = document.querySelector('.md-header,.mast');
    if (header) {
      const measure = () => {
        const height = `${header.getBoundingClientRect().height}px`;
        document.documentElement.style.setProperty('--site-header-height', height);
        if (header.classList.contains('mast')) document.documentElement.style.setProperty('--mast-h', height);
      };
      measure();
      if (typeof ResizeObserver !== 'undefined') new ResizeObserver(measure).observe(header);
      else addEventListener('resize', measure);
    }
    init(document.querySelector('.mast-tabs,.mast-links'), 'a.mast-link,a.md-tabs__link');
    init(document.querySelector('.md-sidebar--primary .md-nav--primary'), 'a.md-nav__link[href]', 'slot-guide-selector');
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, {once:true});
  else start();
})();
