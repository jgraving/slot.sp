// Render the website's actual SP illustration around the current native clip.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const { execFileSync } = require('node:child_process');
const { chromium } = require('playwright');
const repo = path.resolve(__dirname, '..');
const output = path.resolve(process.argv[2]);
const videoPath = path.resolve(process.argv[3] || path.join(repo, 'previews/homepage/hero-light-platinum.mp4'));
fs.mkdirSync(path.join(output, 'model-frames'), { recursive: true });
execFileSync('python3', [path.join(repo, 'previews/extract_model_frames.py'), videoPath, output], { stdio: 'inherit' });
const native = JSON.parse(fs.readFileSync(path.join(output, 'native-frames.json'), 'utf8'));
const home = fs.readFileSync(path.join(repo, 'site/overrides/home.html'), 'utf8');
const start = home.indexOf('<div class="sp-stage">');
const end = home.indexOf('<div class="preview-controls">', start);
if (start < 0 || end < 0) throw Error('Website model markup not found');
const model = home.slice(start, end).replace(/<video[\s\S]*?<\/video>/,
  '<img class="hero-screen screen-box" src="/native-frames/0000.png" alt="">');
const defs = fs.readFileSync(path.join(repo, 'site/overrides/partials/device.svg'), 'utf8');
const fontPath = path.join(repo, 'site/docs/assets/fonts/google-sans-flex.woff2');
if (!fs.existsSync(fontPath)) throw Error('Run tools/prepare-site.py before capturing the README wordmark');
const html = `<!doctype html><html><head><meta charset="utf-8">
<link rel="stylesheet" href="/site/docs/assets/navigation.css">
<link rel="stylesheet" href="/site/docs/assets/device.css">
<link rel="stylesheet" href="/site/docs/assets/slot.css">
<style>html,body{margin:0;width:1000px;height:720px;overflow:hidden;background:#e8e3d8}
*,*::before,*::after{animation:none!important;transition:none!important;box-sizing:border-box}
.readme-brand{position:absolute;left:58px;top:50%;transform:translateY(-50%);width:400px}
.readme-brand .readme-wordmark{display:block;width:max-content;font-size:136px;line-height:1.02;color:#1c2a31}
.readme-brand p{margin:18px 0 0;font:650 34px/1.3 "Google Sans Flex",sans-serif;font-variation-settings:"ROND" 55;letter-spacing:-.045em;color:#1c2a31}
.readme-brand p span{color:#0b6e61}
.readme-model{position:absolute;left:520px;top:0;width:480px;height:720px}
.sp-stage{--dw:300px;margin:46px auto 0}.hero-screen{opacity:1!important}
</style></head><body>${defs}<div class="readme-brand"><span class="mast-mark readme-wordmark">slot.sp</span><p>Your games.<br><span>SP style.</span></p></div><div class="md-typeset readme-model">${model}</div></body></html>`;
const server = http.createServer((req, res) => {
  if (req.url === '/') { res.setHeader('Content-Type', 'text/html'); return res.end(html); }
  const requestPath = new URL(req.url, 'http://localhost').pathname;
  const file = /^\/native-frames\/\d{4}\.png$/.test(requestPath) ? path.join(output, requestPath.slice(1)) : path.resolve(repo, '.' + requestPath);
  if (!file.startsWith(path.join(output, 'native-frames') + path.sep) && !file.startsWith(repo + path.sep)) { res.writeHead(403); return res.end(); }
  try {
    const data = fs.readFileSync(file);
    const type = file.endsWith('.png') ? 'image/png' : file.endsWith('.css') ? 'text/css' : file.endsWith('.woff2') ? 'font/woff2' : 'application/octet-stream';
    res.setHeader('Content-Type', type);
    res.setHeader('Content-Length', data.length);
    res.end(data);
  } catch { res.writeHead(404); res.end(); }
});
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ executablePath: process.env.CHROME_BIN || '/usr/bin/google-chrome', args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  try {
    const page = await browser.newPage({ viewport: { width: 1000, height: 720 }, deviceScaleFactor: 1 });
    await page.goto(`http://127.0.0.1:${server.address().port}/`, { waitUntil: 'networkidle' });
    const wordmark = await page.locator('.readme-wordmark').evaluate(async element => {
      await document.fonts.load('720 136px "Google Sans Flex"', 'slot.sp');
      await document.fonts.ready;
      const style = getComputedStyle(element), rect = element.getBoundingClientRect();
      return { text:element.textContent, family:style.fontFamily, weight:style.fontWeight,
        variation_settings:style.fontVariationSettings, optical_sizing:style.fontOpticalSizing, letter_spacing_em:parseFloat(style.letterSpacing)/parseFloat(style.fontSize),
        loaded:document.fonts.check('720 136px "Google Sans Flex"', 'slot.sp'),
        bounds:{x:rect.x,y:rect.y,right:rect.right,bottom:rect.bottom} };
    });
    if (!wordmark.loaded || !wordmark.family.includes('Google Sans Flex') || wordmark.weight !== '720' ||
        wordmark.variation_settings !== '"ROND" 60, "opsz" 25' || wordmark.optical_sizing !== 'none' || Math.abs(wordmark.letter_spacing_em + .05) > .0001)
      throw Error('README wordmark must use the website mast-mark typography: ' + JSON.stringify(wordmark));
    if (wordmark.bounds.x < 8 || wordmark.bounds.right > 470 || wordmark.bounds.y < 8 || wordmark.bounds.bottom > 712)
      throw Error('README wordmark clipped: ' + JSON.stringify(wordmark.bounds));
    await page.locator('.hero-screen').evaluate(img => img.decode());
    const duration = native.duration;
    if (!(duration > 5 && duration < 20)) throw Error(`Unexpected source clip: ${duration}s`);
    const bounds = await page.locator('.dv-front, .dv-deck').evaluateAll(els => els.map(e => { const r = e.getBoundingClientRect(); return { x:r.x, y:r.y, right:r.right, bottom:r.bottom }; }));
    if (bounds.some(r => r.x < 8 || r.y < 8 || r.right > 992 || r.bottom > 712)) throw Error('Model clipped: ' + JSON.stringify(bounds));
    const frames = native.frames;
    for (let i = 0; i < frames; i++) {
      await page.evaluate(async index => {
        const screen = document.querySelector('.hero-screen');
        screen.src = `/native-frames/${String(index).padStart(4, '0')}.png`;
        await screen.decode();
        await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      }, i);
      await page.screenshot({ path: path.join(output, 'model-frames', `${String(i).padStart(4, '0')}.png`) });
    }
    const nativeManifest = JSON.parse(fs.readFileSync(path.join(repo, 'previews/homepage/provenance.json'), 'utf8'));
    wordmark.font_file = 'site/docs/assets/fonts/google-sans-flex.woff2';
    wordmark.font_sha256 = crypto.createHash('sha256').update(fs.readFileSync(fontPath)).digest('hex');
    wordmark.style_source = 'site/docs/assets/slot.css .mast-mark';
    fs.writeFileSync(path.join(output, 'model-capture.json'), JSON.stringify({ frames, frame_ms:80, duration, bounds, wordmark, browser_version: browser.version(), native_sha256: crypto.createHash('sha256').update(fs.readFileSync(videoPath)).digest('hex'), native_capture_source_commit:process.env.UI_CAPTURE_REV || nativeManifest.capture_source_commit || nativeManifest.source_commit }, null, 2));
    console.log(`Captured ${frames} frames of the website SP model with native playback`);
  } finally { await browser.close(); server.close(); }
})().catch(error => { console.error(error); server.close(); process.exitCode = 1; });
