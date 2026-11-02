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

The homepage keeps its two-column introduction and concise linked feature list. Its handheld is a self-hosted, reference-built 3D model with matching recorded 3D renders while loading and when WebGL is unavailable. Its colors follow the device UI: a dark navy backdrop, charcoal surfaces, teal controls and gold details. Flat pill buttons and rounded surfaces use the same fast spatial spring as the device; color changes do not overshoot. Reduced motion disables these transitions. Its four SP finishes use current dark-theme native clips showing a real insertion, ejection and return to the carousel, plus a matching reduced-motion poster. Updated screenshots and controlled animations illustrate the guide pages. Alternative hero clips load only when selected, while off-screen playback pauses. Guide videos remain under the visitor's control.

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
- The illustrated A/B buttons play the recorded insertion/ejection, the D-pad browses the recorded cartridges, and MENU replays startup. The classic up-up-down-down-left-right-left-right-B-A keyboard code reveals a gold shell. These are optional website details, not advertised device features. They add no motion or sound until a visitor interacts.

Writing references: [NN/G: concise, scannable and objective writing](https://www.nngroup.com/articles/concise-scannable-and-objective-how-to-write-for-the-web/), [NN/G: descriptive links](https://www.nngroup.com/articles/writing-links/), and [GOV.UK: clear language](https://guidance.publishing.service.gov.uk/writing-to-gov-uk-standards/writing-guidelines/clear-language/). [Wikipedia’s AI-writing observations](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) identify patterns to review, not proof of authorship. Judge wording by accuracy and usefulness; punctuation alone is not a reason to rewrite it.

The homepage, guide and Cart Studio share identical `navigation.js`, `navigation.css` and `wordmark.png` assets. Assembly rejects a mismatched Studio pin. Install and Update are separate top-level destinations. The header keeps its navigation on a second row when needed; the guide selector has its own container and saved position. `website-corrections-media` captures Bluetooth-enabled audio examples using fixture-only mutable access in the recorder build; release source and binaries are unaffected.

The Pages build checks global navigation on the homepage, install, update, guide, audio and Studio pages at 1366, 1100, 820, 390 and 320 pixels. It verifies active destinations, hit targets, feature columns, shared header height and opening/closing the guide-topic drawer before publication.

## Homepage hardware model

`site/docs/assets/sp-model.js` reconstructs the RG SP from the supplied opening videos and Anbernic's front/back/left/right/top/bottom product drawings. The official [RG SP product page](https://anbernic.com/products/rg-sp) and its [hinge details](https://anbernic.com/cdn/shop/files/RG_SP-14.webp?v=1785383547&width=1920) establish the 82 × 83 × 23.9 mm closed envelope, 3.4-inch 720×480 screen and metallic painted finishes. This is a hand-built visual model, not a scanned mesh or manufacturing CAD. Personal reference photos and videos are not published.

The five-part hinge, stepped L1/L2/R2/R1 buttons, rear USB-C and mini-HDMI sockets, left volume rocker and TF1 slot, right power/reset controls and TF2 slot, front headphone jack, lanyard openings, sixteen speaker holes and rear screw wells follow the surface views. Reflective micro-facets add a glittery metallic sheen under a satin clear coat; highlights change as the lid and shell move. Shoulder caps extend into recessed shell housings. A baked softbox environment provides broad reflections. Three.js r180 is vendored with its MIT license; the renderer requires no runtime CDN, analytics, remote environment map, or model download.

The screen is 3:2, matching the native 720×480 recording without stretching or letterboxing. A/B, MENU and the D-pad control that recording. All four official finishes and the gold-shell secret recolor the model; the four standard finishes remember the visitor's choice locally. Drag or use arrow keys to inspect every side, then double-click or press Home to reset the view. Hardware hit targets hide when the controls face away. The normal lid opening is 150°, tilted toward the player; the accessible slider runs from closed to exactly 180° flat. The lid button closes it or returns to the playing position. Vertical touch gestures remain available for page scrolling.

The model loads only when the homepage hero approaches the viewport, caps pixel density at 1.75, and stops rendering when the video is paused and the view settles. Hidden and off-screen rendering stops; folding pauses video playback. The model opens from closed to 150° before native playback begins. Reduced motion starts open with a paused poster; manual rotation and folding remain available. Save-Data uses a static render of the same model. Failed WebGL/module loading uses a short recorded 3D opening, then a matching model poster with the native screen projected into its glass and the original HTML controls aligned to the real buttons. Context loss restores that same static view. No SVG illustration or CSS flip is shown during loading or fallback. Guide illustrations and Cart Studio do not load this renderer.

`tools/prepare-site.py` fingerprints the model and its loader together. `site/scripts/check-sp-model.cjs` checks desktop / 390 / 320 pixel layouts, unobstructed hardware controls, the actual insertion endpoint, all four finishes and persistence, the 150° playing position and 180° stop, folding, full rotation and back-facing controls, reduced motion and a deliberately unavailable WebGL context. The Pages workflow retains its rendered screenshots as `sp-model-review` and requires this check before deployment, alongside the existing navigation and link checks.

`tools/capture-readme-model.cjs` renders the production mesh, shaders and camera for all four finishes and the gold secret. `previews/collect_readme_model.py` packages 30 fps H.264 openings, transparent open/closed posters, screen/control projection coordinates, and the matching README GIF. Their provenance includes the exact model hash; preparation rejects stale captures. The browser checks exercise both GPU opening and a deliberately disabled-WebGL recorded opening, including fallback cartridge controls.
