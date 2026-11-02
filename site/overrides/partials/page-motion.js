/* Installed in the head before the first rendering opportunity. */
(() => {
  const root = document.documentElement;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  function direction(activation) {
    root.dataset.pageDirection = activation?.navigationType === 'traverse' &&
      activation.entry?.index < activation.from?.index ? 'back' : 'forward';
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
