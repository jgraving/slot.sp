// Check the real built pages before publication, including narrow headers.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '../site/public');
const output = path.resolve(process.argv[2]);
fs.mkdirSync(output, {recursive:true});
const origin = 'https://jakegraving.com/slot.sp/';
const names = ['Install','Update','Guide','Cart Studio','About'];
const mime = {'.js':'text/javascript','.css':'text/css','.html':'text/html','.wasm':'application/wasm','.woff2':'font/woff2','.svg':'image/svg+xml','.png':'image/png','.webp':'image/webp','.mp4':'video/mp4','.json':'application/json'};
(async () => {
  const browser = await chromium.launch({executablePath:process.env.CHROME_BIN || '/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']});
  const results = [];
  try {
    for (const width of [1366,1100,980,960,959,820,390,320]) {
      const page = await browser.newPage({viewport:{width,height:900},reducedMotion:'reduce',hasTouch:width===980});
      let headerReference;
      await page.route(origin+'**', async route => {
        let file = path.resolve(root, decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
        if (!file.startsWith(root+path.sep) && file!==root) return route.fulfill({status:403});
        if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file=path.join(file,'index.html');
        if (!fs.existsSync(file)) return route.fulfill({status:404,body:'Missing test asset'});
        await route.fulfill({path:file,contentType:mime[path.extname(file)] || 'application/octet-stream'});
      });
      for (const [route,active] of [['',''],['installing/','Install'],['updating/','Update'],['features/guide/','Guide'],['features/bluetooth/','Guide'],['studio/','Cart Studio']]) {
        await page.goto(origin+route, {waitUntil:'domcontentloaded'});
        await page.locator('header .slot-selector[data-selector-ready]').waitFor();
        await page.evaluate(() => document.fonts.ready);
        await page.waitForFunction(() => Math.abs(parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--site-header-height')) - document.querySelector('.md-header,.mast').getBoundingClientRect().height) < 1);
        const info = await page.evaluate(() => {
          const header=document.querySelector('.md-header,.mast'), box=header.getBoundingClientRect();
          const links=[...header.querySelectorAll('.mast-tabs a,.mast-links a')];
          const brand=header.querySelector('.mast-mark').getBoundingClientRect();
          return {headerHeight:box.height,brandCenter:brand.y+brand.height/2,overflow:document.documentElement.scrollWidth>innerWidth+1,
            geometry:{brand:brand.toJSON(),nav:header.querySelector('.mast-tabs,.mast-links').getBoundingClientRect().toJSON(),github:header.querySelector('.mast-gh').getBoundingClientRect().toJSON()},
            links:links.map(link => {const r=link.getBoundingClientRect();return {text:link.textContent.trim(),href:link.href,active:link.hasAttribute('aria-current') || !!link.closest('.md-tabs__item--active'),fits:r.left>=0 && r.right<=innerWidth+1 && r.top>=box.top && r.bottom<=box.bottom+1,hit:document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('a')===link}}),
            overflowItems:[...document.querySelectorAll('body *')].filter(el=>{const r=el.getBoundingClientRect();return r.width && (r.right>innerWidth+1 || r.left< -1)}).slice(0,12).map(el=>({tag:el.tagName,class:el.className,rect:el.getBoundingClientRect().toJSON()})),
            githubHit:(()=>{const link=header.querySelector('.mast-gh'),r=link.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('a')===link;})(),
            searchHit:(()=>{const controls=[...header.querySelectorAll('.md-search__button,label[for="__search"]')],button=controls.find(el=>{const r=el.getBoundingClientRect();return r.width>0&&r.height>0;});if(!controls.length)return null;if(!button)return false;const r=button.getBoundingClientRect();return button.contains(document.elementFromPoint(r.x+r.width/2,r.y+r.height/2));})(),
            palette:getComputedStyle(document.documentElement).getPropertyValue('--page').trim(),
            columns:document.querySelector('.feature-list') ? getComputedStyle(document.querySelector('.feature-list')).gridTemplateColumns.split(/\s+/).length : null,
            headerMeasured:parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--site-header-height'))};
        });
        assert.deepEqual(info.links.map(x=>x.text),names,`${width} ${route}: destinations`);
        assert.deepEqual(info.links.filter(x=>x.active).map(x=>x.text),active?[active]:[],`${width} ${route}: active destination`);
        assert(info.links.every(x=>x.fits && x.hit),`${width} ${route}: header link covered or out of bounds: ${JSON.stringify(info)}`);
        assert(info.githubHit,`${width} ${route}: GitHub control is obscured`);
        if (!headerReference) headerReference=info;
        for (const part of ['brand','nav','github']) for (const property of ['x','y','width','height']) {
          assert(Math.abs(info.geometry[part][property]-headerReference.geometry[part][property])<1,`${width} ${route}: ${part} ${property} must match the homepage: ${JSON.stringify(info.geometry)}`);
        }
        assert(Math.abs(info.headerHeight-headerReference.headerHeight)<1,`${width} ${route}: shared header height`);
        if (route!=='studio/') assert(info.searchHit,`${width} ${route}: search control is obscured`);
        if (width>=960) {
          assert(info.headerHeight<=66,`${width} ${route}: unnecessary second header row (${info.headerHeight}px)`);
          const centers=await page.locator('header .mast-link').evaluateAll(links=>links.map(link=>{const r=link.getBoundingClientRect();return r.y+r.height/2;}));
          assert(centers.every(y=>Math.abs(y-info.brandCenter)<2),`${width} ${route}: navigation must share the brand's row`);
        }
        if (!route) assert.equal(await page.locator('.model-hint').count(),0,'Keep the button interactions unadvertised');
        if(info.overflow) await page.screenshot({path:path.join(output,`${width}-overflow.png`)});
        assert(!info.overflow,`${width} ${route}: horizontal overflow: ${JSON.stringify(info.overflowItems)}`);
        assert.equal(info.palette,'#0e1423');
        assert(Math.abs(info.headerMeasured-info.headerHeight)<1,`${width} ${route}: header measurement`);
        if (info.columns) assert.equal(info.columns,width>960?3:width>540?2:1,`${width}: feature columns`);
        assert(info.links.find(x=>x.text==='Install').href.endsWith('/installing/'));
        assert(info.links.find(x=>x.text==='Update').href.endsWith('/updating/'));
        if (route.startsWith('features/') && width<1220) {
          await page.locator('.md-header label[for="__drawer"]').click();
          assert(await page.locator('#__drawer').isChecked(),`${width}: guide drawer did not open`);
          assert(await page.locator('.md-sidebar--primary .nav-topic').first().isVisible(),`${width}: guide topics unavailable`);
          const overlay=page.locator('label.md-overlay[for="__drawer"]');
          const overlayBox=await overlay.boundingBox();
          assert(overlayBox && overlayBox.width>0,`${width}: guide overlay unavailable`);
          await overlay.click({position:{x:overlayBox.width-12,y:Math.min(450,overlayBox.height/2)}});
          assert(!(await page.locator('#__drawer').isChecked()),`${width}: guide drawer did not close`);
        }
        results.push({width,route,...info});
        if ([1366,980,390].includes(width) && ['', 'features/guide/', 'studio/'].includes(route)) await page.screenshot({path:path.join(output,`${width}-${route.replaceAll('/','-')||'home'}.png`)});
      }
      await page.close();
    }
    fs.writeFileSync(path.join(output,'navigation.json'),JSON.stringify(results,null,2));
    console.log('PASS: 48 page/viewport combinations; compact desktop headers, Android desktop width, five global destinations, active tabs, feature columns, unobstructed links and guide drawers');
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1});
