// Render the website's actual SP illustration around the current native clip.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const { chromium } = require('playwright');
const repo = path.resolve(__dirname, '..');
const output = path.resolve(process.argv[2]);
const videoPath = path.resolve(process.argv[3] || path.join(repo, 'previews/homepage/hero-platinum.mp4'));
fs.mkdirSync(path.join(output, 'model-frames'), { recursive: true });
const home = fs.readFileSync(path.join(repo, 'site/overrides/home.html'), 'utf8');
const start = home.indexOf('<div class="sp-stage">');
const end = home.indexOf('<div class="preview-controls">', start);
if (start < 0 || end < 0) throw Error('Website model markup not found');
const model = home.slice(start, end).replace(/<video[\s\S]*?<\/video>/,
  '<video class="hero-screen screen-box" muted playsinline preload="auto" src="/native.mp4"></video>');
const defs = fs.readFileSync(path.join(repo, 'site/overrides/partials/device.svg'), 'utf8');
const html = `<!doctype html><html><head><meta charset="utf-8">
<link rel="stylesheet" href="/site/docs/assets/device.css">
<link rel="stylesheet" href="/site/docs/assets/slot.css">
<style>html,body{margin:0;width:600px;height:760px;overflow:hidden;background:#13171e}
*,*::before,*::after{animation:none!important;transition:none!important;box-sizing:border-box}
.sp-stage{--dw:320px;margin:46px auto 0}.hero-screen{opacity:1!important}
</style></head><body>${defs}<div class="md-typeset">${model}</div></body></html>`;
const server = http.createServer((req, res) => {
  if (req.url === '/') { res.setHeader('Content-Type', 'text/html'); return res.end(html); }
  const file = req.url === '/native.mp4' ? videoPath : path.resolve(repo, '.' + new URL(req.url, 'http://localhost').pathname);
  if (file !== videoPath && !file.startsWith(repo + path.sep)) { res.writeHead(403); return res.end(); }
  try {
    const data = fs.readFileSync(file);
    const type = file.endsWith('.mp4') ? 'video/mp4' : file.endsWith('.css') ? 'text/css' : 'application/octet-stream';
    res.setHeader('Content-Type', type);
    res.setHeader('Content-Length', data.length);
    res.end(data);
  } catch { res.writeHead(404); res.end(); }
});
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ executablePath: process.env.CHROME_BIN || '/usr/bin/google-chrome', args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  try {
    const page = await browser.newPage({ viewport: { width: 600, height: 760 }, deviceScaleFactor: 1 });
    await page.goto(`http://127.0.0.1:${server.address().port}/`, { waitUntil: 'networkidle' });
    await page.waitForFunction(() => document.querySelector('video').readyState >= 2);
    const duration = await page.evaluate(() => { const v = document.querySelector('video'); v.pause(); return v.duration; });
    if (!(duration > 10 && duration < 20)) throw Error(`Unexpected source clip: ${duration}s`);
    const bounds = await page.locator('.dv-front, .dv-deck').evaluateAll(els => els.map(e => { const r = e.getBoundingClientRect(); return { x:r.x, y:r.y, right:r.right, bottom:r.bottom }; }));
    if (bounds.some(r => r.x < 8 || r.y < 8 || r.right > 592 || r.bottom > 752)) throw Error('Model clipped: ' + JSON.stringify(bounds));
    const frames = Math.round(duration * 1000 / 80);
    for (let i = 0; i < frames; i++) {
      await page.evaluate(async time => {
        const v = document.querySelector('video');
        if (Math.abs(v.currentTime - time) > .001) await new Promise(resolve => { v.addEventListener('seeked', resolve, { once: true }); v.currentTime = time; });
        await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      }, Math.min(i * .08, duration - .02));
      await page.screenshot({ path: path.join(output, 'model-frames', `${String(i).padStart(4, '0')}.png`) });
    }
    fs.writeFileSync(path.join(output, 'model-capture.json'), JSON.stringify({ frames, frame_ms:80, duration, bounds, native_sha256: crypto.createHash('sha256').update(fs.readFileSync(videoPath)).digest('hex') }, null, 2));
    console.log(`Captured ${frames} frames of the website SP model with native playback`);
  } finally { await browser.close(); server.close(); }
})().catch(error => { console.error(error); server.close(); process.exitCode = 1; });
