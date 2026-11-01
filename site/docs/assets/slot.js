(() => {
  const script = new URL(document.currentScript.src);
  const assets = new URL(script.pathname.includes('/generated/') ? '../' : './', script);
  const themeKey = 'slot.sp.site-theme';
  const preference = matchMedia('(prefers-color-scheme: dark)');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const topics = {guide:'guide','getting-started':'getting-started',controls:'controls',library:'library',playing:'playing','kid-mode':'kid-mode',bluetooth:'bluetooth',display:'display','night-display':'night-display',screenshots:'screenshots','power-options':'power-options','link-play':'link-play','cart-studio':'cart-studio',settings:'settings',storage:'storage',customizing:'cart-studio',troubleshooting:'help'};
  const finishes = {
    graphite:['#5a5d64','#34363b','#16171a','#3c3e43','#5c5f66','rgba(255,255,255,.42)'],
    platinum:['#e2e5e9','#b8bcc2','#7e838b','#c2c6cc','#e9ecf0','rgba(0,0,0,.46)'],
    'pearl blue':['#dbe9f2','#a8c4d8','#6d8ba1','#d0dae3','#edf3f8','rgba(0,0,0,.44)'],
    'pearl pink':['#f6d3dd','#e6a6b8','#a76d7e','#dbd1d5','#f2eaed','rgba(0,0,0,.44)']
  };
  function topicIcon(name, className) {
    const icon = document.createElement('span');
    icon.style.setProperty('--icon', `url("${new URL(`icons/${name}.svg`, assets).href}")`);
    icon.setAttribute('aria-hidden', 'true');
    icon.className = className;
    return icon;
  }
  function init() {
    document.querySelectorAll('.md-sidebar--primary a.md-nav__link').forEach(link => {
      const topic = new URL(link.href).pathname.split('/').filter(Boolean).at(-1);
      if (topics[topic] && !link.querySelector('.nav-topic-icon')) {link.classList.add('nav-topic');link.prepend(topicIcon(topics[topic], 'nav-topic-icon'));}
    });
    const topic = location.pathname.split('/').filter(Boolean).at(-1);
    const heading = document.querySelector('article h1');
    if (topics[topic] && heading) heading.prepend(topicIcon(topics[topic], 'page-topic-icon'));

    const preview = document.getElementById('hero-preview');
    const toggle = document.querySelector('.preview-toggle');
    let wanted = !reduced.matches;
    let visible = false;
    let finish = 'platinum';
    let request = 0;
    let heroVariant = '';
    let theme = document.documentElement.dataset.uiTheme || (preference.matches ? 'dark' : 'light');
    const explicitTheme = () => {try {return localStorage.getItem(themeKey);} catch {return null;}};
    function state() {
      if (!toggle) return;
      toggle.setAttribute('aria-pressed', String(wanted));
      toggle.querySelector('span').textContent = wanted ? 'Pause preview' : 'Play preview';
    }
    function sync() {
      if (!preview) return;
      if (wanted && visible && !document.hidden) {const current = request;preview.play().catch(error => {if (current === request && error.name !== 'AbortError') {wanted = false;state();}});}
      else preview.pause();
    }
    function changeHero() {
      if (!preview) return;
      const variant = `${theme}-${finish}`;
      if (variant === heroVariant) return;
      heroVariant = variant;
      const media = new URL(preview.dataset.media, document.baseURI);
      const position = preview.readyState > 0 ? preview.currentTime : 0;
      const current = ++request;
      preview.pause();
      preview.poster = new URL(`hero-${variant}.png`, media).href;
      preview.setAttribute('aria-label', `The current ${theme} interface: cartridge carousel and startup animation, ${finish.replaceAll('_',' ')} finish`);
      preview.addEventListener('loadedmetadata', () => {
        if (current !== request) return;
        preview.currentTime = Math.min(position, Math.max(0, preview.duration - .01));
        sync();
      }, {once:true});
      preview.src = new URL(`hero-${variant}.mp4`, media).href;
      preview.load();
      sync();
    }
    function applyTheme(next, save = false) {
      theme = next === 'dark' ? 'dark' : 'light';
      document.documentElement.dataset.uiTheme = theme;
      document.body.dataset.mdColorScheme = theme === 'dark' ? 'slate' : 'default';
      if (save) {try {localStorage.setItem(themeKey,theme);} catch {}}
      const themeButton = document.querySelector('.site-theme-toggle');
      if (themeButton) {themeButton.setAttribute('aria-label', `Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`);themeButton.title = themeButton.getAttribute('aria-label');}
      document.querySelectorAll('button[data-ui-theme]').forEach(button => button.setAttribute('aria-pressed',String(button.dataset.uiTheme === theme)));
      document.querySelectorAll('[data-theme-source]').forEach(source => {source.media = theme === 'dark' ? 'all' : 'not all';});
      document.querySelectorAll('video[data-theme-video]').forEach(video => {
        const media = new URL(video.dataset.media, document.baseURI);
        const src = new URL(`${theme}/${video.dataset.scene}.mp4`,media).href;
        if (video.src === src) return;
        const position = video.readyState > 0 ? video.currentTime : 0;
        const playing = !video.paused;
        video.poster = new URL(`${theme}/${video.dataset.scene}.png`,media).href;
        video.src = src;
        video.addEventListener('loadedmetadata', () => {video.currentTime = Math.min(position,Math.max(0,video.duration-.01));if (playing && !document.hidden) video.play().catch(() => {});}, {once:true});
        video.load();
      });
      changeHero();
    }
    document.querySelector('.site-theme-toggle')?.addEventListener('click',() => applyTheme(theme === 'dark' ? 'light' : 'dark', true));
    document.querySelectorAll('button[data-ui-theme]').forEach(button => button.addEventListener('click',() => applyTheme(button.dataset.uiTheme,true)));
    preference.addEventListener('change',() => {if (!explicitTheme()) applyTheme(preference.matches ? 'dark' : 'light');});
    document.querySelectorAll('[data-shell]').forEach(button => button.addEventListener('click',() => {
      const values = finishes[button.dataset.shell];
      ['--shell-light','--shell-base','--shell-shade','--btn','--btn-hi','--etch'].forEach((key,i) => document.documentElement.style.setProperty(key,values[i]));
      document.querySelectorAll('[data-shell]').forEach(other => other.setAttribute('aria-pressed',String(other === button)));
      finish = button.dataset.shell.replaceAll(' ','_');
      changeHero();
    }));
    applyTheme(theme);
    if (preview && toggle) {
      toggle.addEventListener('click',() => {wanted = !wanted;sync();state();});
      reduced.addEventListener('change',() => {if (reduced.matches) {wanted = false;sync();state();}});
      new IntersectionObserver(entries => {visible = entries[0].isIntersecting;sync();},{threshold:.1}).observe(preview);
      document.addEventListener('visibilitychange',sync);
      state();
    }
    document.addEventListener('visibilitychange',() => {if (document.hidden) document.querySelectorAll('video').forEach(video => video.pause());});
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded',init,{once:true});
  else init();
})();
