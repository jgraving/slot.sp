/* Keep document navigation and app lifecycles intact on every browser. */
(() => {
  const root = document.documentElement;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const native = 'onpageswap' in window && 'onpagereveal' in window;
  root.dataset.pageMotion = native ? 'native' : 'fallback';
  function section(url) {
    if (!url) return null;
    const path = new URL(url).pathname.replace(/^\/slot\.sp\/?/, '').split('/')[0];
    return !path ? 0 : path === 'installing' ? 1 : path === 'updating' ? 2 :
      path === 'features' || path === 'guide' ? 3 : path === 'studio' ? 4 : 5;
  }
  if (section(location.href) === 3) root.dataset.pageSection = 'guide';
  function direction(from, to, backwards = false) {
    const a = section(from), b = section(to);
    root.toggleAttribute('data-guide-transition', a === 3 && b === 3);
    root.dataset.pageDirection = a !== null && b !== null && a !== b ?
      (b < a ? 'back' : 'forward') : backwards ? 'back' : 'forward';
  }
  function ready() {
    delete root.dataset.pageTransition;
    delete root.dataset.pageFallback;
    root.removeAttribute('data-guide-transition');
    document.dispatchEvent(new Event('slot:page-ready'));
  }
  if (native) {
    // Opt in only where cross-document transitions exist. An older browser
    // must use the fallback rather than silently losing all page movement.
    const style = document.createElement('style');
    style.textContent = '@media(prefers-reduced-motion:no-preference){@view-transition{navigation:auto}}';
    document.head.append(style);
    const orient = activation => direction(activation?.from?.url, activation?.entry?.url,
      activation?.navigationType === 'traverse' && activation.entry?.index < activation.from?.index);
    addEventListener('pageswap', event => {
      orient(event.activation);
      if (reduced.matches) event.viewTransition?.skipTransition();
    });
    addEventListener('pagereveal', event => {
      orient(window.navigation?.activation);
      if (!event.viewTransition) { ready(); return; }
      if (reduced.matches) { event.viewTransition.skipTransition(); ready(); return; }
      root.dataset.pageTransition = 'active';
      event.viewTransition.finished.then(ready, ready);
    });
    return;
  }

  const departureKey = 'slot-page-departure-v2';
  const read = () => { try { return JSON.parse(sessionStorage.getItem(departureKey)); } catch { return null; } };
  const previous = read();
  const oldState = history.state && typeof history.state === 'object' ? history.state : {};
  const index = Number.isFinite(oldState.slotPageIndex) ? oldState.slotPageIndex : (previous?.index || 0) + 1;
  try { history.replaceState({...oldState, slotPageIndex:index}, ''); } catch {}
  const local = url => url.origin === location.origin && url.pathname.startsWith('/slot.sp/');
  const from = previous?.url || document.referrer;
  const type = performance.getEntriesByType('navigation')[0]?.type;
  let generation = 0, exits = [];
  const layers = () => [...document.querySelectorAll(root.hasAttribute('data-guide-transition') ? '.md-content__inner' : '.md-container,body>.wrap,body>.site-foot')];
  const cancelExits = () => { exits.forEach(animation => animation.cancel()); exits = []; };
  function incoming(source) {
    if (reduced.matches) { ready(); return; }
    direction(source?.url || from, location.href, source?.index > index);
    root.dataset.pageTransition = 'active';
    root.dataset.pageFallback = 'incoming';
  }
  function watchIncoming() {
    if (root.dataset.pageFallback !== 'incoming') return;
    // Focus/history restoration can retain a horizontal offset from a moving
    // document. These layouts have no horizontal scroll; keep the header fixed.
    if (scrollX) scrollTo({left:0,top:scrollY,behavior:'instant'});
    const current = generation;
    requestAnimationFrame(() => {
      const animations = document.getAnimations?.().filter(animation => animation.animationName === 'slot-fallback-in') || [];
      if (animations.length) Promise.allSettled(animations.map(animation => animation.finished)).then(() => {
        if (current === generation) ready();
      });
      else if (current === generation) ready();
    });
  }
  if (type !== 'reload' && from && local(new URL(from)) && new URL(from).href !== location.href) incoming(previous);
  addEventListener('DOMContentLoaded', watchIncoming, {once:true});
  addEventListener('pagehide', () => {
    try { sessionStorage.setItem(departureKey, JSON.stringify({url:location.href, index, at:Date.now()})); } catch {}
  });
  addEventListener('pageshow', event => {
    if (!event.persisted) return;
    ++generation; cancelExits();
    delete root.dataset.pageFallback;
    // Restart the entry spring on a restored document, including its native app.
    void document.documentElement.offsetWidth;
    incoming(read()); watchIncoming();
  });

  const warmed = new Map();
  function destination(event) {
    const link = event.target.closest?.('a[href]');
    if (!link || link.hasAttribute('download') || (link.target && link.target !== '_self')) return null;
    const url = new URL(link.href, location.href);
    if (!local(url) || /\.[^/]+$/.test(url.pathname) ||
        (url.pathname === location.pathname && url.search === location.search)) return null;
    return url;
  }
  function warm(url) {
    if (!warmed.has(url.href)) {
      const promise = fetch(url.href, {credentials:'same-origin', cache:'force-cache'}).then(async response => {
        if (!response.ok || !response.headers.get('content-type')?.includes('text/html')) return;
        const doc = new DOMParser().parseFromString(await response.text(), 'text/html');
        await Promise.allSettled([...doc.querySelectorAll('link[rel=stylesheet][href]')].map(link => {
          const css = new URL(link.getAttribute('href'), url);
          return css.origin === location.origin ? fetch(css.href, {cache:'force-cache'}) : Promise.resolve();
        }));
      }).catch(() => {});
      warmed.set(url.href, promise);
      if (warmed.size > 8) warmed.delete(warmed.keys().next().value);
    }
    return warmed.get(url.href);
  }
  addEventListener('pointerover', event => {
    if (reduced.matches || !event.target.closest?.('header a[href]')) return;
    const url = destination(event); if (url) warm(url);
  }, {passive:true});
  addEventListener('click', async event => {
    if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || reduced.matches) return;
    const url = destination(event); if (!url) return;
    event.preventDefault();
    const current = ++generation;
    direction(location.href, url.href);
    root.dataset.pageTransition = 'active';
    // Warm HTML and styles while the old content remains visible. Scripts and
    // Studio still run once in their own document; no partial app replacement.
    await Promise.race([warm(url), new Promise(resolve => setTimeout(resolve, 800))]);
    if (current !== generation) return;
    if (!Element.prototype.animate) { location.assign(url.href); return; }
    const elements = layers(), start = elements.map(element => getComputedStyle(element).transform);
    cancelExits();
    root.dataset.pageFallback = 'leaving';
    document.dispatchEvent(new Event('slot:page-leaving'));
    const distance = Math.min(112, Math.max(48, innerWidth * .08)) * (root.dataset.pageDirection === 'back' ? 1 : -1);
    exits = elements.map((element, i) => element.animate([
      {transform:start[i]}, {transform:`translate3d(${distance}px,0,0)`}
    ], {duration:reduced.matches ? 0 : 180, easing:'cubic-bezier(.4,0,1,1)', fill:'forwards'}));
    await Promise.allSettled(exits.map(animation => animation.finished));
    if (current === generation) location.assign(url.href);
  });
  reduced.addEventListener('change', () => {
    if (reduced.matches) { cancelExits(); ready(); }
  });
})();
