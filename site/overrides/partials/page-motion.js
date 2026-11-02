/* Installed in the head before the first rendering opportunity. */
(() => {
  const root = document.documentElement;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  function section(url) {
    if (!url) return null;
    const path = new URL(url).pathname.replace(/^\/slot\.sp\/?/, '').split('/')[0];
    if (!path) return 0;
    if (path === 'installing') return 1;
    if (path === 'updating') return 2;
    if (path === 'features' || path === 'guide') return 3;
    if (path === 'studio') return 4;
    return 5;
  }
  function direction(activation) {
    const from = section(activation?.from?.url), to = section(activation?.entry?.url);
    const backwards = from !== null && to !== null && from !== to ? to < from :
      activation?.navigationType === 'traverse' && activation.entry?.index < activation.from?.index;
    root.dataset.pageDirection = backwards ? 'back' : 'forward';
  }
  addEventListener('pageswap', event => {
    direction(event.activation);
    if (reduced.matches) event.viewTransition?.skipTransition();
  });
  addEventListener('pagereveal', event => {
    direction(window.navigation?.activation);
    delete root.dataset.pageTransition;
    if (!event.viewTransition) return;
    if (reduced.matches) { event.viewTransition.skipTransition(); return; }
    root.dataset.pageTransition = 'active';
    const settled = () => {
      delete root.dataset.pageTransition;
      document.dispatchEvent(new Event('slot:page-ready'));
    };
    event.viewTransition.finished.then(settled, settled);
  });
})();
