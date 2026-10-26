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
- Copies reviewed native H.264 clips and PNG screenshots, verifying their recorded SHA-256 hashes. GitHub uses the corresponding GIFs; the website uses videos with play controls.
- Builds a self-hosted Google Sans Flex WOFF2 from the source pinned in the UI font manifest, verifying its SHA-256 first. Its license is included with the website font.

`site/studio.json` pins the revision of `jgraving/slot-cart-studio` published under `/slot.sp/studio/`. The assembly step checks that source revision and its renderer pin, includes the app license, and records both in `studio/build-info.json`. Update the pin after merging a tested change in the Studio repository.

Edit the root Markdown guides to update feature instructions. Generated pages, previews and fonts are ignored by Git. The homepage and shared layout live in `site/overrides/`; the fork's colors and typography live in `site/docs/assets/slot.css`.

## Publish

The `pages` workflow builds and checks pull requests. Merging a website or guide change into `main` rebuilds and deploys the site automatically. It can also run manually from Actions.

Repository **Settings → Pages → Source** must be **GitHub Actions**. No custom domain or deployment token is required. The inherited upstream `CNAME` is deliberately absent.

Checks cover the strict documentation build, local page links and anchors, CSS assets, project subpath, self-hosted font references, 720×480 H.264 previews with a decoded first frame and the pinned Studio WebAssembly package.

## Captures

The **Capture documentation media** workflow on `docs/media-refresh` builds the production UI and recorder, runs the relevant native regressions, and exports the README clips and guide screenshots. The `documentation-media` artifact contains the media, provenance and prepared Git blob entries. Review the images before committing those objects with the GitHub integration. The workflow does not create a commit or move a branch.

`previews/media-provenance.json` records the source commit, Actions run, file sizes and hashes. Menu examples use explicit sample settings. These are software-rendered examples, not photographs or hardware test results.

The **Capture homepage previews** workflow records the production frontend browsing all five fixture cartridges and renders the glass logo sequence in all four SP finishes. `previews/compose_homepage.py` joins them with transitions back into the carousel. The README uses the same Platinum sequence playing inside the website’s SP model. `tools/capture-readme-model.cjs` captures the shared model markup, CSS perspective and native clip, with a matching reduced-motion poster. The homepage capture workflow regenerates this illustrated hero alongside the screen clips; the **Capture illustrated README hero** workflow can refresh it independently on `docs/readme-model`. `previews/homepage/provenance.json` records the clips, posters and GIF. The homepage picker switches both the illustrated handheld and the native preview, preserving playback position and the visitor's play/pause choice. It downloads each alternative clip only when selected. Reduced motion keeps a still poster until playback is requested.

## Layout and copy

- Give installation, the guide and Studio a clear place in navigation. Keep one maintained guide per topic; preserve superseded URLs with redirects.
- Use the actual interface beside the steps it explains. Give clips play controls and keep them quiet until requested.
- Name the feature and say what it does. Avoid slogans, repeated download sections and lists of vague benefits.
- Keep the SP illustration and recognizable feature icons. Use the same symbols in the homepage and guide navigation so topics are easy to distinguish. Favor readable labels and space over extra small captions.
- Use teal for links and actions, with Google Sans Flex for both the site and device UI.
