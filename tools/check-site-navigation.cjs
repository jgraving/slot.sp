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
const mime = {'.js':'text/javascript','.css':'text/css','.html':'text/html','.wasm':'application/wasm','.woff2':'font/woff2','.svg':'image/svg+xml','.png':'image/png','.mp4':'video/mp4','.json':'application/json'};
(async () => {
  const browser = await chromium.launch({executablePath:process.env.CHROME_BIN || '/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']});
  const results = [];
  try {
    for (const width of [1366,1100,820,390,320]) {
      const page = await browser.newPage({viewport:{width,height:900},reducedMotion:'reduce'});
      await page.route(origin+'**', async route => {
        let file = path.resolve(root, decodeURIComponent(new URL(route.request().url()).pathname.slice('/slot.sp/'.length)));
        if (!file.startsWith(root+path.sep) && file!==root) return route.fulfill({status:403});
        if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file=path.join(file,'index.html');
        if (!fs.existsSync(file)) return route.fulfill({status:404,body:'Missing test asset'});
        await route.fulfill({path:file,contentType:mime[path.extname(file)] || 'application/octet-stream'});
      });
      for (const [route,active] of [['',''],['installing/','Install'],['updating/','Update'],['features/guide/','Guide'],['studio/','Cart Studio']]) {
        await page.goto(origin+route, {waitUntil:'domcontentloaded'});
        await page.locator('header .slot-selector[data-selector-ready]').waitFor();
        await page.evaluate(() => document.fonts.ready);
        const info = await page.evaluate(() => {
          const header=document.querySelector('.md-header,.mast'), box=header.getBoundingClientRect();
          const links=[...header.querySelectorAll('.mast-tabs a,.mast-links a')];
          return {headerHeight:box.height,overflow:document.documentElement.scrollWidth>innerWidth+1,
            links:links.map(link => {const r=link.getBoundingClientRect();return {text:link.textContent.trim(),href:link.href,active:link.matches('[aria-current="page"]') || !!link.closest('.md-tabs__item--active'),fits:r.left>=0 && r.right<=innerWidth+1 && r.top>=box.top && r.bottom<=box.bottom+1,hit:document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('a')===link}}),
            palette:getComputedStyle(document.documentElement).getPropertyValue('--page').trim(),
            columns:document.querySelector('.feature-list') ? getComputedStyle(document.querySelector('.feature-list')).gridTemplateColumns.split(/\s+/).length : null,
            headerMeasured:parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--site-header-height'))};
        });
        assert.deepEqual(info.links.map(x=>x.text),names,`${width} ${route}: destinations`);
        assert.deepEqual(info.links.filter(x=>x.active).map(x=>x.text),active?[active]:[],`${width} ${route}: active destination`);
        assert(info.links.every(x=>x.fits && x.hit),`${width} ${route}: header link covered or out of bounds: ${JSON.stringify(info)}`);
        assert(!info.overflow,`${width} ${route}: horizontal overflow`);
        assert.equal(info.palette,'#0e1423');
        assert(Math.abs(info.headerMeasured-info.headerHeight)<1,`${width} ${route}: header measurement`);
        if (info.columns) assert.equal(info.columns,width>960?3:width>540?2:1,`${width}: feature columns`);
        assert(info.links.find(x=>x.text==='Install').href.endsWith('/installing/'));
        assert(info.links.find(x=>x.text==='Update').href.endsWith('/updating/'));
        if (route==='features/guide/' && width<1220) {
          await page.locator('.md-header label[for="__drawer"]').click();
          assert(await page.locator('#__drawer').isChecked(),`${width}: guide drawer did not open`);
          assert(await page.locator('.md-sidebar--primary .nav-topic').first().isVisible(),`${width}: guide topics unavailable`);
          await page.locator('label.md-overlay[for="__drawer"]').click();
          assert(!(await page.locator('#__drawer').isChecked()),`${width}: guide drawer did not close`);
        }
        results.push({width,route,...info});
        if ((width===1366 || width===390) && ['', 'features/guide/', 'studio/'].includes(route)) await page.screenshot({path:path.join(output,`${width}-${route.replaceAll('/','-')||'home'}.png`)});
      }
      await page.close();
    }
    fs.writeFileSync(path.join(output,'navigation.json'),JSON.stringify(results,null,2));
    console.log('PASS: 25 page/viewport combinations; five global destinations, active tabs, feature columns, unobstructed links and guide drawers');
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1});
