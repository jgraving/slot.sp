# slot.sp website

Published at **https://jakegraving.com/slot.sp/** through GitHub Pages.

## Build

Use Python 3.13, Rust 1.96 with the `wasm32-unknown-unknown` target, and wasm-pack 0.15.0. From the repository root:

```sh
python -m venv .venv-site
.venv-site/bin/pip install zensical==0.0.71 'fonttools[woff]==4.61.1' pillow==13.0.0 --only-binary=av av==16.0.0
mkdir -p _studio
git clone https://github.com/jgraving/slot-cart-studio.git _studio/slot-cart-studio
git -C _studio/slot-cart-studio checkout "$(python -c 'import json; print(json.load(open("site/studio.json"))["ref"])')"
git clone https://github.com/BrandonKowalski/slot.git _studio/slot
git -C _studio/slot checkout "$(cat _studio/slot-cart-studio/slot.ref)"
(cd _studio/slot-cart-studio && wasm-pack build --release --target web --no-typescript --out-dir web/pkg)
.venv-site/bin/python tools/prepare-site.py
.venv-site/bin/zensical build --clean --strict -f site/zensical.build.toml
.venv-site/bin/python tools/assemble-studio.py
.venv-site/bin/python tools/check-site.py
```

The output is `site/public/`. The preparation step:

- Turns the root feature guides into website pages, adapting relative links and heading anchors.
- Creates `site/zensical.build.toml` with content-hashed CSS and JavaScript URLs. The source configuration stays in `site/zensical.toml`; build the generated configuration after preparing the site.
- Copies the reviewed current native captures in both themes, verifying their recorded SHA-256 hashes. GitHub uses a few compact GIFs; the website uses H.264 videos with play controls. Each capture set is served under its source revision to avoid stale media after updates.
- Builds a self-hosted Google Sans Flex WOFF2 from the source pinned in the UI font manifest, verifying its SHA-256 first. Its license is included with the website font.

`site/studio.json` pins the revision of `jgraving/slot-cart-studio` published under `/slot.sp/studio/`. The assembly step checks that source revision and its renderer pin, includes the app license, and records both in `studio/build-info.json`. Update the pin after merging a tested change in the Studio repository.

Edit the root Markdown guides to update feature instructions. Generated pages, previews and fonts are ignored by Git. The homepage and shared layout live in `site/overrides/`; the fork's colors and typography live in `site/docs/assets/slot.css`.

## Publish

The `pages` workflow builds and checks pull requests. Merging a website or guide change into `main` rebuilds and deploys the site automatically. It can also run manually from Actions.

Repository **Settings → Pages → Source** must be **GitHub Actions**. No custom domain or deployment token is required. The inherited upstream `CNAME` is deliberately absent.

Checks cover the strict documentation build, local page links and anchors, CSS assets, project subpath, self-hosted font references, 720×480 H.264 previews with a decoded first frame and the pinned Studio WebAssembly package.

## Captures

The **Capture current website media** workflow on `docs/website-ui-refresh` runs `previews/site_refresh.rs` through the production Frontend/compositor, then `previews/collect_site_refresh.py`. It records both UI themes, settings and parental controls, software gameplay examples, charging, startup motion, and eight matching theme/finish hero clips. The fixture uses repository label artwork and synthetic games, never a playable ROM. The gallery has current button hints and native captures in both themes. Save-state and link examples use Dark mode: their legacy Light controls still need a separate app contrast fix. Low-contrast Light captures are excluded rather than recolored.

`previews/current/provenance.json` records the capture source, UI release, rendering method, sample data, byte sizes and SHA-256 hashes. `previews/homepage/provenance.json` tracks the eight hero variants. The `current-website-media` artifact contains media and prepared Git blob entries for review; capture itself never creates a commit or moves a branch.

The homepage keeps its two-column introduction and concise linked feature list. Its handheld uses a rotatable, reference-built 3D model with a native screen texture and accessible HTML buttons. Matching opening and closing clips provide a fixed-angle fallback when WebGL is unavailable. The same reserved layout is present from first paint through playback. Its colors follow the device UI: a dark navy backdrop, charcoal surfaces, teal controls and gold details. Flat pill buttons and rounded surfaces use the same fast spatial spring as the device; color changes do not overshoot. Reduced motion disables these transitions. Its four SP finishes use current native UI actions with five selectable cartridges, real insertion and ejection, and matching reduced-motion stills. Updated screenshots and controlled animations illustrate the guide pages. Alternative hero clips load only when selected, while off-screen playback pauses. Guide videos remain under the visitor's control.

The README uses the same production 3D model, flips it open, and plays the Platinum light-theme clip on its screen, with the current UI’s rounded Google Sans Flex display lettering, built by `tools/build-web-wordmark.py` from the pinned `display.ttf` profile. `tools/capture-readme-model.cjs` and `previews/collect_readme_model.py` preserve a matching poster and record provenance. The prior capture scripts remain available for specialist asset work; the website publishes only the current reviewed manifest.

## Layout and copy

- Give installation, the guide and Studio a clear place in navigation. Keep one maintained guide per topic; preserve superseded URLs with redirects.
- Use the actual interface beside the steps it explains. Give clips play controls and keep them quiet until requested.
- Name each feature and explain what someone can do with it. Use the actual menu labels for steps. Explain terms such as save state or bezel when they first appear.
- Keep all existing feature topics represented. Add missing features to the existing summary and link to their guides; use the guides for detailed steps and screenshots.
- Use short, factual sentences and descriptive links. Remove vague praise, repetitive claims, rhetorical contrasts and filler. Keep necessary detail even when it makes a page longer.
- Keep actions and navigation familiar. Do not add decorative buttons, arrows, badges, card grids or screenshot sections to make the page seem new.
- Keep the recognizable SP silhouette and feature icons. Use the same symbols in the homepage and guide navigation so topics are easy to distinguish. Favor readable labels and space over extra small captions.
- Keep one visual system across the homepage and guide: device colors, neutral body type, rounded headings and flat controls. Preserve the main page structure and SP preview. Use concise feature summaries; keep detailed instructions in the guide.
- The homepage plays startup once, then demonstrates browsing, insertion and ejection until a hardware button is pressed. Manual controls follow the OS: Left/Right browse, Up/Down jump initial letters, A inserts and holding MENU for one second returns to the carousel. B does not eject or reboot; menu-only actions have no substitute behavior. The single GBA shelf does not implement system switching, Favorites, kid approval, settings or emulation. Rapid browsing preserves native motion. The classic up-up-down-down-left-right-left-right-B-A keyboard code still reveals a gold shell.

Writing references: [NN/G: concise, scannable and objective writing](https://www.nngroup.com/articles/concise-scannable-and-objective-how-to-write-for-the-web/), [NN/G: descriptive links](https://www.nngroup.com/articles/writing-links/), and [GOV.UK: clear language](https://guidance.publishing.service.gov.uk/writing-to-gov-uk-standards/writing-guidelines/clear-language/). [Wikipedia’s AI-writing observations](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) identify patterns to review, not proof of authorship. Judge wording by accuracy and usefulness; punctuation alone is not a reason to rewrite it.

The homepage, guide and bundled Cart Studio load the exact same fingerprinted navigation assets. Studio assembly preserves its pinned app and artwork while taking header layout and navigation behavior from the website build. Install and Update are separate top-level destinations. The centered header uses one grid on every page, stays on one row from 960 pixels wide, including Android desktop mode, and wraps on smaller screens. Guide topics sit beside search without shifting the brand or tabs; the guide selector has its own container and saved position. `website-corrections-media` captures Bluetooth-enabled audio examples using fixture-only mutable access in the recorder build; release source and binaries are unaffected.

The Pages build checks global navigation on the homepage, install, update, guide, audio and Studio pages at 1366, 1100, 980, 960, 959, 820, 390 and 320 pixels. It verifies active destinations, hit targets, feature columns, shared header height and opening/closing the guide-topic drawer before publication.

## Homepage hardware model

`site/docs/assets/sp-model.js` reconstructs the RG SP from the supplied opening videos and Anbernic's front/back/left/right/top/bottom product drawings. The official [RG SP product page](https://anbernic.com/products/rg-sp) and its [hinge details](https://anbernic.com/cdn/shop/files/RG_SP-14.webp?v=1785383547&width=1920) establish the 82 × 83 × 23.9 mm closed envelope, 3.4-inch 720×480 screen and metallic painted finishes. This is a hand-built visual model, not a scanned mesh or manufacturing CAD. Personal reference photos and videos are not published.

The five-part hinge, stepped L1/L2/R2/R1 buttons, rear USB-C and mini-HDMI sockets, left volume rocker and TF1 slot, right power/reset controls and TF2 slot, front headphone jack, lanyard openings, sixteen speaker holes and rear screw wells follow the surface views. Reflective micro-facets add a glittery metallic sheen under a satin clear coat; highlights change as the lid and shell move. Shoulder caps extend into recessed shell housings. A baked softbox environment provides broad reflections. Three.js r180 is vendored with its MIT license; the renderer requires no runtime CDN, analytics, remote environment map, or model download.

The screen is 3:2, matching the native 720×480 recordings. The painted canvas is uploaded to the model as a texture, so the screen is occluded by the lid and chassis and stays on the glass through rotation. The same projected button targets move with their meshes; rear buttons use their own face normals. Drag to rotate, use arrow keys while the model has focus, and double-click or press Home to restore the original angled view. With a hardware button focused, arrow keys control the OS preview. A single lid button opens to the 150° playing position or closes it without resetting the cartridge.

The live model replaces its matching transparent poster only after a complete render at the same camera and reserved stage size. No crossfade, blend mode or colored matte is applied to it. The screen, shell, finish controls and wordmark keep their layout throughout initial loading. Rendering sleeps when idle, hidden or offscreen; pointer and lid motion follow display refresh, while native screen updates repaint at up to 30 Hz. The current finish alone loads its compressed native film. Reduced motion and Save-Data suppress startup and autoplay and use native stills for manual actions.

The L1/R1 caps wrap over the outer rear corners, with long skirts extending down the chassis. L2/R2 are narrower inset caps, slightly higher, with shorter skirts. A narrow dark reveal replaces the previous raised mounting blocks. These shapes follow the official rear, side and angled product references, including the [shoulder detail](https://anbernic.com/cdn/shop/files/RG_SP-14.webp?v=1785383547&width=1920).

If WebGL is unavailable or its context is lost, the existing fixed-angle rendered fallback retains the native player and lid controls. `tools/capture-readme-model.cjs` captures matching open/closed posters, opening frames, button projections and rear/side review views. Collection packages these with matching closing clips and source hashes. Preparation requires the model source hash to match its fallback capture, preventing stale shoulder geometry or button positions.

`site/scripts/check-sp-model.cjs` covers the renderer handoff, complete silhouette, drag and keyboard rotation, all shoulder targets, actual boot frames, automatic slotting, first-button takeover, all five cartridges, wrapping, rapid reversals, hold-to-exit, lid suspension, finish persistence and narrow layouts. `check-sp-video.cjs` explicitly disables WebGL to retain the existing Chromium/WebKit pixel comparisons for the video fallback. Navigation, Studio and link checks still run before deployment.

## Page transitions

The website and bundled Cart Studio share opaque horizontal page slides based on Material 3 Expressive springs. Pages follow the order of the navigation destinations; history within a section follows Back/Forward direction. The header stays anchored. Native cross-document transitions move the full page instead of fading overlapping content, and the active pill remains a separate opaque shared element. Labels remain visible above it. Local hover and keyboard selection use an analytic spring that preserves velocity when retargeted.

The motion parameters follow AndroidX Material3's `ExpressiveMotionTokens.kt`: fast spatial (stiffness 800, damping .6) for the pill, and slow spatial (200, .8) for the page. Browsers without cross-document support receive a document-navigation fallback: warm the destination HTML and styles, slide the outgoing content, and spring the incoming content into place. The header selector resumes from its saved visible position before the first paint. Studio and documentation scripts still initialize in their own documents; there is no partial app replacement. Back/Forward and section anchors retain normal browser behavior. Reduced motion disables movement, and the homepage lid intro waits for the incoming slide to finish.

`tools/check-page-motion.cjs` follows 42 native and fallback transitions through installation, updates, Cart Studio, About, guides and Home at desktop, Android desktop and phone widths. It checks actual travel, opaque content, pill visibility, Back/Forward, anchors, reduced motion and the lid handoff. Real-time fallback screencasts include the document handoff rather than assembling only paused animation frames. Navigation checks compare brand, tab and GitHub geometry against Home on every route.

The lid video renders normally and remains opaque across the model. Only the empty outer margin is feathered into the page. There is no blend mode, color filter or per-frame pixel processing. The still remains visible until the first presented frame, then hides during playback to prevent a ghost lid. `site/scripts/check-sp-video.cjs` compares dark model, screen and shadow pixels with the actual decoded source in 48 frames across all four finishes, both directions and Chromium/WebKit, including first load with a saved Pearl Blue finish. Matching edges alone cannot catch lost shading or a translucent device.
## Interactive homepage actions

`previews/interactive_hero.rs` records the production Frontend, carousel springs, cartridge occlusion and boot compositor. The disposable capture checkout exposes App stepping and fixes the wallpaper phase; it never drives an emulator. These recorder-only changes are made by `prepare_interactive_capture.py`, are not committed to the device source, and do not enter release builds. Five synthetic headers use the repository sample artwork. Each color has one 60 Hz H.264 film, a compact WebP strip of settled poses, and a frame-indexed timeline. Collection checks every action join and records source identity and hashes.

`hero-player.js` paints frames into a 720×480 canvas, used as a WebGL texture or projected into the rendered fallback. A decoded frame stays visible while the video seeks; the player never exposes an unrelated segment. A bounded destination and desired slot state preserve rapid presses and reversals. The current finish is loaded on demand; reduced motion and Save-Data use only static native poses. No menu UI, WebAssembly runtime or game core is loaded. Autoplay uses the same bounded action queue as manual input. Its timer runs only while visible, open and active; any hardware button permanently takes control until the visitor explicitly presses Play preview. The current clip reaches a matching endpoint before queued input continues. A manually inserted cartridge stays seated until MENU is held. Browser checks cover boot ordering, automatic browsing/slotting, takeover, all five cartridges, wrapping, rapid input, keyboard controls, lid suspension and state continuity.
