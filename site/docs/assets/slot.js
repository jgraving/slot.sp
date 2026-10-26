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
        const values = finishes[button.dataset.shell];
        ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,values[i]));
        document.querySelectorAll('[data-shell]').forEach(other => other.setAttribute('aria-pressed',String(other === button)));
        changePreview(button);
      });
    });

    const preview = document.getElementById('hero-preview');
    const toggle = document.querySelector('.preview-toggle');
    if (!preview || !toggle) return;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    let wanted = !reduced.matches;
    let visible = true;
    let finish = 'platinum';
    let request = 0;
    let position = 0;
    const media = new URL(preview.dataset.media, document.baseURI);
    function changePreview(button) {
      const next = button.dataset.shell.replaceAll(' ', '_');
      if (next === finish) return;
      finish = next;
      if (preview.readyState > 0) position = preview.currentTime;
      const current = ++request;
      preview.pause();
      preview.poster = new URL(`hero-${finish}.png`, media).href;
      preview.setAttribute('aria-label', `Cartridge insertion and removal in slot.sp, ${button.getAttribute('aria-label')} finish`);
      preview.addEventListener('loadedmetadata', () => {
        if (current !== request) return;
        preview.currentTime = Math.min(position, Math.max(0, preview.duration - .01));
        sync();
      }, {once:true});
      preview.src = new URL(`hero-${finish}.mp4`, media).href;
      preview.load();
      state();
    }
    function sync() {
      if (wanted && visible && !document.hidden) preview.play().catch(() => {});
      else preview.pause();
    }
    function state() {
      toggle.setAttribute('aria-pressed', String(wanted));
      toggle.querySelector('span').textContent = wanted ? 'Pause preview' : 'Play preview';
    }
    preview.addEventListener('play',state);
    preview.addEventListener('pause',state);
    toggle.addEventListener('click',() => { wanted = !wanted; sync(); state(); });
    reduced.addEventListener('change',() => { if (reduced.matches) { wanted = false; sync(); state(); } });
    new IntersectionObserver(entries => { visible = entries[0].isIntersecting; sync(); },{threshold:.1}).observe(preview);
    document.addEventListener('visibilitychange',sync);
    state();
  }
  document.addEventListener('visibilitychange',() => {
    if (document.hidden) document.querySelectorAll('video').forEach(video => video.pause());
  });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded',init,{once:true});
  else init();
})();
